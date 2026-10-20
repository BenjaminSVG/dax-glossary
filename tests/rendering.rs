use std::collections::BTreeMap;

use dax_glossary::{catalog, RenderError};

fn p(pairs: &[(&str, &str)]) -> BTreeMap<String, String> {
    pairs.iter().map(|(k, v)| (k.to_string(), v.to_string())).collect()
}

#[test]
fn simple_aggregation() {
    let out = catalog()
        .render_measure("sum", &p(&[("measure_name", "Total Sales"), ("column", "'Sales'[Amount]")]))
        .unwrap();
    assert_eq!(out, "Total Sales = SUM('Sales'[Amount])");
}

#[test]
fn ytd_and_fiscal_ytd_with_text_quoting() {
    let c = catalog();
    let out = c
        .render_measure(
            "ytd",
            &p(&[("measure_name", "Sales YTD"), ("measure", "[Sales]"), ("dates", "'Date'[Date]")]),
        )
        .unwrap();
    assert_eq!(out, "Sales YTD = TOTALYTD([Sales], 'Date'[Date])");
    let out = c
        .render_measure(
            "ytd-fiscal",
            &p(&[
                ("measure_name", "FY YTD"),
                ("measure", "[Sales]"),
                ("dates", "'Date'[Date]"),
                ("year_end", "6/30"),
            ]),
        )
        .unwrap();
    assert_eq!(out, "FY YTD = TOTALYTD([Sales], 'Date'[Date], \"6/30\")");
}

#[test]
fn yoy_percent_is_multiline_with_name_on_its_own_line() {
    let out = catalog()
        .render_measure(
            "yoy-pct",
            &p(&[("measure_name", "Sales YoY %"), ("measure", "[Sales]"), ("dates", "'Date'[Date]")]),
        )
        .unwrap();
    assert!(
        out.starts_with(
            "Sales YoY % =\nVAR _prior = CALCULATE([Sales], SAMEPERIODLASTYEAR('Date'[Date]))\nRETURN DIVIDE([Sales] - _prior, _prior)"
        ),
        "{out}"
    );
}

#[test]
fn top_n_uses_number_parameter() {
    let out = catalog()
        .render_measure(
            "top-n-sum",
            &p(&[
                ("measure_name", "Top 5"),
                ("measure", "[Sales]"),
                ("column", "'Product'[Name]"),
                ("n", "5"),
            ]),
        )
        .unwrap();
    assert_eq!(out, "Top 5 = SUMX(TOPN(5, ALLSELECTED('Product'[Name]), [Sales], DESC), [Sales])");
}

#[test]
fn calculation_item_renders_as_bare_expression() {
    let out = catalog()
        .render_measure("calc-item-ytd", &p(&[("measure_name", "YTD"), ("dates", "'Date'[Date]")]))
        .unwrap();
    assert_eq!(out, "CALCULATE(SELECTEDMEASURE(), DATESYTD('Date'[Date]))");
}

#[test]
fn text_parameters_escape_double_quotes() {
    let out = catalog()
        .render_measure(
            "title-text",
            &p(&[
                ("measure_name", "T"),
                ("label", "Say \"hi\""),
                ("measure", "[Sales]"),
                ("format", "\"#,0\""),
                ("column", "'Product'[Category]"),
            ]),
        )
        .unwrap();
    assert!(out.contains("\"Say \"\"hi\"\"\" & \": \""), "{out}");
}

#[test]
fn missing_and_extra_parameters_are_reported() {
    let c = catalog();
    match c.render_measure("sum", &p(&[("measure_name", "X")])) {
        Err(RenderError::MissingParameters(m)) => assert_eq!(m, vec!["column".to_string()]),
        other => panic!("{other:?}"),
    }
    match c.render_measure("sum", &p(&[("measure_name", "X"), ("column", "'T'[C]"), ("bogus", "1")])) {
        Err(RenderError::UnexpectedParameters(e)) => assert_eq!(e, vec!["bogus".to_string()]),
        other => panic!("{other:?}"),
    }
    assert!(matches!(
        c.render_measure("unknown-id", &p(&[])),
        Err(RenderError::UnknownTemplate(_))
    ));
}

#[test]
fn values_are_validated_per_type() {
    let c = catalog();
    let bad = |pairs: &[(&str, &str)]| c.render_measure("sum", &p(pairs)).unwrap_err();
    // column without table, unterminated quote, injection attempts
    assert!(matches!(
        bad(&[("measure_name", "X"), ("column", "[Amount]")]),
        RenderError::InvalidValue { .. }
    ));
    assert!(matches!(
        bad(&[("measure_name", "X"), ("column", "'Sales[Amount]")]),
        RenderError::InvalidValue { .. }
    ));
    assert!(matches!(
        bad(&[("measure_name", "X"), ("column", "Sales[Amount]) + 1 + (")]),
        RenderError::InvalidValue { .. }
    ));
    assert!(matches!(
        bad(&[("measure_name", "X = 1"), ("column", "Sales[Amount]")]),
        RenderError::InvalidValue { .. }
    ));
    assert!(matches!(
        bad(&[("measure_name", "  "), ("column", "Sales[Amount]")]),
        RenderError::InvalidValue { .. }
    ));
    // numbers must be finite numbers
    let n = |v: &str| {
        c.render_measure(
            "top-n-sum",
            &p(&[("measure_name", "X"), ("measure", "[S]"), ("column", "T[C]"), ("n", v)]),
        )
    };
    assert!(n("5").is_ok());
    assert!(n("5); DROP").is_err());
    assert!(n("NaN").is_err());
    // measures are [Name] only, expressions must be balanced
    assert!(c
        .render_measure(
            "ratio",
            &p(&[("measure_name", "R"), ("numerator", "Sales"), ("denominator", "[D]")])
        )
        .is_err());
    let f = |v: &str| {
        c.render_measure(
            "filtered-value",
            &p(&[("measure_name", "F"), ("measure", "[S]"), ("column", "T[C]"), ("value", v)]),
        )
    };
    assert!(f("\"Red\"").is_ok());
    assert!(f("\"Red\") + (1").is_err());
    assert!(f("\"unterminated").is_err());
}

#[test]
fn substituted_values_are_never_re_expanded() {
    // a value that looks like a placeholder must come out verbatim
    let out = catalog()
        .render_measure(
            "filtered-value",
            &p(&[
                ("measure_name", "F"),
                ("measure", "[S]"),
                ("column", "T[C]"),
                ("value", "\"{{measure}}\""),
            ]),
        )
        .unwrap();
    assert_eq!(out, "F = CALCULATE([S], T[C] = \"{{measure}}\")");
}

#[test]
fn every_template_example_renders_without_leftover_placeholders() {
    let c = catalog();
    for m in c.measures() {
        let ex: BTreeMap<String, String> = m.parameters.iter().map(|x| (x.name.clone(), x.example.clone())).collect();
        let out = c.render_measure(&m.id, &ex).unwrap_or_else(|e| panic!("{}: {e}", m.id));
        assert!(!out.contains("{{") && !out.contains("}}"), "{}: {out}", m.id);
    }
}
