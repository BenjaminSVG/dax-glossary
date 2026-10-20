use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fmt;
use std::sync::OnceLock;

use serde::{Deserialize, Serialize};

const FUNCTIONS_JSON: &str = include_str!("../data/functions.json");
const MEASURES_JSON: &str = include_str!("../data/measures.json");
const COVERAGE_JSON: &str = include_str!("../data/coverage.json");

pub const FUNCTION_CATEGORIES: &[&str] = &[
    "Aggregation",
    "Calculation group",
    "Date and time",
    "Filter",
    "Financial",
    "INFO",
    "Information",
    "Logical",
    "Math and trig",
    "Other",
    "Parent and child",
    "Relationship",
    "Statistical",
    "Table manipulation",
    "Text",
    "Time intelligence",
    "Visual calculations",
    "Window",
];
pub const MEASURE_CATEGORIES: &[&str] = &[
    "Basic",
    "Calculation item",
    "Filter",
    "Formatting",
    "Ranking",
    "Share",
    "Statistics",
    "Time intelligence",
];

#[derive(Debug)]
pub enum Error {
    /// The embedded data could not be parsed (a bug in the crate's data files).
    Data(String),
    UnknownFunction(String),
    UnknownMeasure(String),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Error::Data(m) => write!(f, "catalog data error: {m}"),
            Error::UnknownFunction(n) => write!(f, "unknown DAX function '{n}'"),
            Error::UnknownMeasure(n) => write!(f, "unknown measure template '{n}'"),
        }
    }
}
impl std::error::Error for Error {}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arg {
    pub name: String,
    pub required: bool,
    pub variadic: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub category: String,
    pub tags: Vec<String>,
    pub syntax: String,
    #[serde(default)]
    pub syntax_alternatives: Vec<String>,
    pub args: Vec<Arg>,
    /// number | text | boolean | date | datetime | table | column | scalar | variant | none
    pub returns: String,
    pub description: String,
    pub reference_url: String,
    /// "microsoft-learn" when the syntax was taken from the Learn page, else "catalog".
    pub syntax_source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ParamKind {
    Table,
    Column,
    Measure,
    Expression,
    Name,
    Number,
    Text,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Parameter {
    pub name: String,
    #[serde(rename = "type")]
    pub kind: ParamKind,
    pub description: String,
    pub example: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MeasureKind {
    Measure,
    CalculationItem,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Measure {
    pub id: String,
    pub name: String,
    pub kind: MeasureKind,
    pub category: String,
    pub description: String,
    pub body: String,
    pub parameters: Vec<Parameter>,
    pub assumptions: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Coverage {
    pub review_date: String,
    pub source: String,
    pub method: String,
    pub function_count: usize,
    pub measure_template_count: usize,
    pub categories: BTreeMap<String, usize>,
    pub listed_on_category_pages: usize,
    pub category_pages_missing_from_catalog: Vec<String>,
    pub catalog_functions_not_on_category_pages: Vec<String>,
    pub notes: Vec<String>,
}

/// One search result: either a function or a measure template.
#[derive(Debug, Clone, Serialize)]
pub struct SearchHit<'a> {
    pub score: u32,
    pub function: Option<&'a Function>,
    pub measure: Option<&'a Measure>,
}

#[derive(Debug)]
pub struct Catalog {
    functions: Vec<Function>,
    measures: Vec<Measure>,
    coverage: Coverage,
    fn_index: BTreeMap<String, usize>,
    ms_index: BTreeMap<String, usize>,
}

static CATALOG: OnceLock<Catalog> = OnceLock::new();

/// The embedded catalog, parsed once per process.
pub fn catalog() -> &'static Catalog {
    CATALOG.get_or_init(|| Catalog::load().expect("embedded dax-glossary data must be valid"))
}

impl Catalog {
    pub fn load() -> Result<Catalog, Error> {
        let mut functions: Vec<Function> = serde_json::from_str(FUNCTIONS_JSON).map_err(|e| Error::Data(format!("functions.json: {e}")))?;
        let mut measures: Vec<Measure> = serde_json::from_str(MEASURES_JSON).map_err(|e| Error::Data(format!("measures.json: {e}")))?;
        let coverage: Coverage = serde_json::from_str(COVERAGE_JSON).map_err(|e| Error::Data(format!("coverage.json: {e}")))?;
        functions.sort_by(|a, b| a.name.cmp(&b.name));
        measures.sort_by(|a, b| a.id.cmp(&b.id));
        let fn_index = functions
            .iter()
            .enumerate()
            .map(|(i, f)| (f.name.to_ascii_uppercase(), i))
            .collect();
        let ms_index = measures.iter().enumerate().map(|(i, m)| (m.id.to_ascii_lowercase(), i)).collect();
        Ok(Catalog {
            functions,
            measures,
            coverage,
            fn_index,
            ms_index,
        })
    }

    pub fn functions(&self) -> &[Function] {
        &self.functions
    }
    pub fn measures(&self) -> &[Measure] {
        &self.measures
    }
    pub fn coverage(&self) -> &Coverage {
        &self.coverage
    }

    /// Case-insensitive lookup by function name (`"totalytd"` finds `TOTALYTD`).
    pub fn get_function(&self, name: &str) -> Result<&Function, Error> {
        self.fn_index
            .get(&name.trim().to_ascii_uppercase())
            .map(|&i| &self.functions[i])
            .ok_or_else(|| Error::UnknownFunction(name.to_string()))
    }

    pub fn get_measure(&self, id: &str) -> Result<&Measure, Error> {
        self.ms_index
            .get(&id.trim().to_ascii_lowercase())
            .map(|&i| &self.measures[i])
            .ok_or_else(|| Error::UnknownMeasure(id.to_string()))
    }

    /// Function categories that have at least one function, sorted.
    pub fn function_categories(&self) -> Vec<&str> {
        self.functions
            .iter()
            .map(|f| f.category.as_str())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    pub fn measure_categories(&self) -> Vec<&str> {
        self.measures
            .iter()
            .map(|m| m.category.as_str())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }

    /// Functions of a category (case-insensitive), in name order.
    pub fn functions_in(&self, category: &str) -> Vec<&Function> {
        self.functions
            .iter()
            .filter(|f| f.category.eq_ignore_ascii_case(category.trim()))
            .collect()
    }

    pub fn measures_in(&self, category: &str) -> Vec<&Measure> {
        self.measures
            .iter()
            .filter(|m| m.category.eq_ignore_ascii_case(category.trim()))
            .collect()
    }

    /// Functions carrying a tag (e.g. `iterator`, `volatile`, `calculate-modifier`).
    pub fn functions_tagged(&self, tag: &str) -> Vec<&Function> {
        self.functions
            .iter()
            .filter(|f| f.tags.iter().any(|t| t.eq_ignore_ascii_case(tag)))
            .collect()
    }

    /// Free-text search over functions and measure templates. Every word must match somewhere; best matches first, ties by name.
    pub fn search(&self, query: &str) -> Vec<SearchHit<'_>> {
        let words: Vec<String> = query.split_whitespace().map(|w| w.to_lowercase()).collect();
        if words.is_empty() {
            return Vec::new();
        }
        let mut hits = Vec::new();
        for f in &self.functions {
            let name = f.name.to_lowercase();
            let hay = format!("{} {} {} {}", f.description, f.category, f.tags.join(" "), f.returns).to_lowercase();
            if let Some(score) = score(&words, &name, &hay) {
                hits.push(SearchHit {
                    score,
                    function: Some(f),
                    measure: None,
                });
            }
        }
        for m in &self.measures {
            let name = format!("{} {}", m.id, m.name).to_lowercase();
            let hay = format!("{} {} {}", m.description, m.category, m.body).to_lowercase();
            if let Some(score) = score(&words, &name, &hay) {
                hits.push(SearchHit {
                    score,
                    function: None,
                    measure: Some(m),
                });
            }
        }
        hits.sort_by(|a, b| b.score.cmp(&a.score).then_with(|| key(a).cmp(&key(b))));
        hits
    }

    /// Integrity report: an empty list means the data is consistent.
    pub fn validate(&self) -> Vec<String> {
        let mut issues = Vec::new();
        let mut seen = HashSet::new();
        for f in &self.functions {
            if !seen.insert(f.name.to_ascii_uppercase()) {
                issues.push(format!("duplicate function {}", f.name));
            }
            if !FUNCTION_CATEGORIES.contains(&f.category.as_str()) {
                issues.push(format!("{}: unknown category '{}'", f.name, f.category));
            }
            if f.description.trim().is_empty() {
                issues.push(format!("{}: empty description", f.name));
            }
            if !f.reference_url.starts_with("https://learn.microsoft.com/") {
                issues.push(format!("{}: reference_url must be a Microsoft Learn link", f.name));
            }
            let up = f.syntax.to_ascii_uppercase();
            let n = f.name.to_ascii_uppercase();
            if !(up.starts_with(&format!("{n}(")) || up.starts_with(&format!("{n} ("))) {
                issues.push(format!("{}: syntax must start with the function name", f.name));
            }
            let (mut p, mut b) = (0i32, 0i32);
            for c in f.syntax.chars() {
                match c {
                    '(' => p += 1,
                    ')' => p -= 1,
                    '[' => b += 1,
                    ']' => b -= 1,
                    _ => {}
                }
            }
            if p != 0 || b != 0 {
                issues.push(format!("{}: unbalanced brackets in syntax", f.name));
            }
            for a in f.args.iter().filter(|a| a.required) {
                if !f.syntax.contains(&format!("<{}>", a.name)) {
                    issues.push(format!("{}: argument '{}' not found in syntax", f.name, a.name));
                }
            }
        }
        let mut ids = HashSet::new();
        for m in &self.measures {
            if !ids.insert(m.id.to_ascii_lowercase()) {
                issues.push(format!("duplicate measure id {}", m.id));
            }
            if !MEASURE_CATEGORIES.contains(&m.category.as_str()) {
                issues.push(format!("{}: unknown category '{}'", m.id, m.category));
            }
            if m.description.trim().is_empty() || m.body.trim().is_empty() {
                issues.push(format!("{}: empty description or body", m.id));
            }
            let declared: HashSet<&str> = m.parameters.iter().map(|p| p.name.as_str()).collect();
            if declared.len() != m.parameters.len() {
                issues.push(format!("{}: duplicate parameter names", m.id));
            }
            let used: HashSet<String> = crate::render::placeholders(&m.body).into_iter().collect();
            for ph in &used {
                if !declared.contains(ph.as_str()) {
                    issues.push(format!("{}: placeholder {{{{{ph}}}}} is not a declared parameter", m.id));
                }
            }
            for p in &m.parameters {
                if p.name != "measure_name" && !used.contains(&p.name) {
                    issues.push(format!("{}: parameter '{}' is never used", m.id, p.name));
                }
                if p.example.trim().is_empty() {
                    issues.push(format!("{}: parameter '{}' has no example", m.id, p.name));
                }
            }
            let ex: BTreeMap<String, String> = m.parameters.iter().map(|p| (p.name.clone(), p.example.clone())).collect();
            match self.render_measure(&m.id, &ex) {
                Ok(s) if s.contains("{{") => issues.push(format!("{}: rendered example still has placeholders", m.id)),
                Ok(_) => {}
                Err(e) => issues.push(format!("{}: example does not render: {e}", m.id)),
            }
        }
        if self.coverage.function_count != self.functions.len() {
            issues.push("coverage.json function_count differs from functions.json".into());
        }
        if self.coverage.measure_template_count != self.measures.len() {
            issues.push("coverage.json measure_template_count differs from measures.json".into());
        }
        issues
    }

    pub fn functions_json(&self) -> String {
        serde_json::to_string_pretty(&self.functions).expect("serializable")
    }

    pub fn measures_json(&self) -> String {
        serde_json::to_string_pretty(&self.measures).expect("serializable")
    }

    pub fn functions_csv(&self) -> String {
        let mut out = String::from("name,category,returns,tags,syntax,description,reference_url\n");
        for f in &self.functions {
            let tags = f.tags.join(";");
            out.push_str(&csv_row(&[
                &f.name,
                &f.category,
                &f.returns,
                &tags,
                &f.syntax,
                &f.description,
                &f.reference_url,
            ]));
        }
        out
    }

    pub fn measures_csv(&self) -> String {
        let mut out = String::from("id,name,kind,category,description,parameters,assumptions,body\n");
        for m in &self.measures {
            let params = m
                .parameters
                .iter()
                .map(|p| format!("{}:{}", p.name, kind_str(p.kind)))
                .collect::<Vec<_>>()
                .join(";");
            let kind = match m.kind {
                MeasureKind::Measure => "measure",
                MeasureKind::CalculationItem => "calculation_item",
            };
            out.push_str(&csv_row(&[
                &m.id,
                &m.name,
                kind,
                &m.category,
                &m.description,
                &params,
                &m.assumptions,
                &m.body,
            ]));
        }
        out
    }
}

pub(crate) fn kind_str(k: ParamKind) -> &'static str {
    match k {
        ParamKind::Table => "table",
        ParamKind::Column => "column",
        ParamKind::Measure => "measure",
        ParamKind::Expression => "expression",
        ParamKind::Name => "name",
        ParamKind::Number => "number",
        ParamKind::Text => "text",
    }
}

fn key(h: &SearchHit<'_>) -> String {
    h.function
        .map(|f| f.name.clone())
        .or_else(|| h.measure.map(|m| m.id.clone()))
        .unwrap_or_default()
}

fn score(words: &[String], name: &str, hay: &str) -> Option<u32> {
    let mut total = 0;
    for w in words {
        total += if name == w {
            100
        } else if name.starts_with(w.as_str()) {
            60
        } else if name.contains(w.as_str()) {
            40
        } else if hay.contains(w.as_str()) {
            10
        } else {
            return None;
        };
    }
    Some(total)
}

fn csv_row(cells: &[&str]) -> String {
    let mut s = cells.iter().map(|c| csv_cell(c)).collect::<Vec<_>>().join(",");
    s.push('\n');
    s
}

fn csv_cell(c: &str) -> String {
    if c.contains([',', '"', '\n', '\r']) {
        format!("\"{}\"", c.replace('"', "\"\""))
    } else {
        c.to_string()
    }
}
