use dax_glossary::{catalog, Catalog};

#[test]
fn embedded_data_passes_every_integrity_check() {
    let issues = catalog().validate();
    assert!(issues.is_empty(), "integrity problems:\n{}", issues.join("\n"));
}

#[test]
fn covers_every_function_listed_on_the_microsoft_category_pages() {
    let cov = catalog().coverage();
    assert!(
        cov.category_pages_missing_from_catalog.is_empty(),
        "missing: {:?}",
        cov.category_pages_missing_from_catalog
    );
    assert!(cov.function_count >= 380, "only {} functions", cov.function_count);
    assert!(cov.listed_on_category_pages >= 380);
}

#[test]
fn well_known_functions_exist_with_expected_shape() {
    let c = catalog();
    for name in [
        "SUM",
        "CALCULATE",
        "TOTALYTD",
        "SUMMARIZECOLUMNS",
        "RANKX",
        "DATEADD",
        "SELECTEDMEASURE",
        "WINDOW",
        "INFO.TABLES",
        "RUNNINGSUM",
        "XIRR",
        "DIVIDE",
        "EARLIER",
    ] {
        assert!(c.get_function(name).is_ok(), "{name} missing");
    }
    let sum = c.get_function("sum").unwrap();
    assert_eq!(sum.category, "Aggregation");
    assert_eq!(sum.returns, "number");
    assert_eq!(sum.args.len(), 1);
    assert!(sum.args[0].required && !sum.args[0].variadic);
    assert!(c
        .get_function("TOTALYTD")
        .unwrap()
        .tags
        .contains(&"requires-date-table".to_string()));
    assert!(c.get_function("SUMX").unwrap().tags.contains(&"iterator".to_string()));
    assert!(c.get_function("NOW").unwrap().tags.contains(&"volatile".to_string()));
}

#[test]
fn lookup_is_case_insensitive_and_reports_unknown_names() {
    let c = catalog();
    assert_eq!(c.get_function(" totalytd ").unwrap().name, "TOTALYTD");
    assert_eq!(c.get_function("Info.Tables").unwrap().name, "INFO.TABLES");
    assert!(c.get_function("NOT_A_FUNCTION").is_err());
    assert!(c.get_measure("nope").is_err());
}

#[test]
fn every_category_has_functions_and_filtering_is_case_insensitive() {
    let c = catalog();
    assert_eq!(c.function_categories().len(), 18);
    assert!(!c.functions_in("time INTELLIGENCE").is_empty());
    assert!(c.functions_in("no such category").is_empty());
    let total: usize = c.function_categories().iter().map(|cat| c.functions_in(cat).len()).sum();
    assert_eq!(total, c.functions().len());
}

#[test]
fn search_ranks_exact_names_first_and_requires_every_word() {
    let c = catalog();
    let hits = c.search("sumx");
    assert_eq!(hits[0].function.unwrap().name, "SUMX");
    assert!(c
        .search("year to date")
        .iter()
        .any(|h| h.function.is_some_and(|f| f.name == "TOTALYTD")));
    assert!(c.search("zzzz qqqq").is_empty());
    assert!(c.search("   ").is_empty());
    // deterministic order
    let a: Vec<_> = c.search("rank").iter().map(|h| h.score).collect();
    let b: Vec<_> = c.search("rank").iter().map(|h| h.score).collect();
    assert_eq!(a, b);
}

#[test]
fn load_is_repeatable_and_ordered() {
    let a = Catalog::load().unwrap();
    let b = Catalog::load().unwrap();
    assert_eq!(a.functions_json(), b.functions_json());
    assert!(a.functions().windows(2).all(|w| w[0].name < w[1].name));
}

#[test]
fn exports_are_deterministic_and_csv_escapes_correctly() {
    let c = catalog();
    let csv = c.functions_csv();
    assert!(csv.starts_with("name,category,returns,tags,syntax,description,reference_url\n"));
    assert_eq!(csv.lines().filter(|l| l.starts_with("SUM,")).count(), 1);
    // syntax contains commas, so it must be quoted
    assert!(csv.contains("\"TOTALYTD("));
    assert_eq!(csv, c.functions_csv());
    let mcsv = c.measures_csv();
    assert!(mcsv.starts_with("id,name,kind,category,description,parameters,assumptions,body\n"));
    let parsed: serde_json::Value = serde_json::from_str(&c.functions_json()).unwrap();
    assert_eq!(parsed.as_array().unwrap().len(), c.functions().len());
}
