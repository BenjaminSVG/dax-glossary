use std::process::{Command, Output};

fn run(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_dax-glossary"))
        .args(args)
        .output()
        .expect("binary runs")
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).to_string()
}

#[test]
fn list_functions_by_category() {
    let o = run(&["list", "functions", "--category", "time intelligence"]);
    assert!(o.status.success());
    let s = stdout(&o);
    assert!(s.contains("TOTALYTD") && s.contains("SAMEPERIODLASTYEAR") && !s.contains("SUMX"));
}

#[test]
fn list_measures_and_categories() {
    assert!(stdout(&run(&["list", "measures"])).contains("yoy-pct"));
    let cats = stdout(&run(&["categories", "functions"]));
    assert!(cats.contains("Aggregation") && cats.contains("Window"));
    assert!(stdout(&run(&["categories", "measures"])).contains("Share"));
}

#[test]
fn get_function_text_and_json() {
    let o = run(&["get", "function", "calculate"]);
    assert!(o.status.success() && stdout(&o).contains("CALCULATE("));
    let j: serde_json::Value = serde_json::from_str(&stdout(&run(&["get", "function", "SUM", "--json"]))).unwrap();
    assert_eq!(j["name"], "SUM");
    assert_eq!(j["category"], "Aggregation");
}

#[test]
fn get_measure() {
    let o = run(&["get", "measure", "ytd"]);
    assert!(o.status.success());
    assert!(stdout(&o).contains("--param dates=<column>"));
}

#[test]
fn search_finds_and_fails_with_nonzero_exit() {
    let o = run(&["search", "year", "to", "date"]);
    assert!(o.status.success() && stdout(&o).contains("TOTALYTD"));
    let o = run(&["search", "zzzzqqqq"]);
    assert!(!o.status.success());
    assert!(String::from_utf8_lossy(&o.stderr).contains("nothing found"));
}

#[test]
fn render_text_and_json() {
    let args = [
        "render",
        "ytd",
        "--param",
        "measure_name=Sales YTD",
        "--param",
        "measure=[Sales]",
        "--param",
        "dates='Date'[Date]",
    ];
    let o = run(&args);
    assert!(o.status.success());
    assert_eq!(stdout(&o).trim(), "Sales YTD = TOTALYTD([Sales], 'Date'[Date])");
    let mut with_json = args.to_vec();
    with_json.push("--json");
    let j: serde_json::Value = serde_json::from_str(&stdout(&run(&with_json))).unwrap();
    assert_eq!(j["dax"], "Sales YTD = TOTALYTD([Sales], 'Date'[Date])");
}

#[test]
fn render_errors_go_to_stderr_with_exit_code_2() {
    let o = run(&["render", "sum", "--param", "measure_name=X"]);
    assert_eq!(o.status.code(), Some(2));
    assert!(String::from_utf8_lossy(&o.stderr).contains("missing parameter"));
    assert!(stdout(&o).is_empty());
    let o = run(&["render", "sum", "--param", "novalue"]);
    assert_eq!(o.status.code(), Some(2));
}

#[test]
fn export_json_and_csv_to_stdout_and_file() {
    let j: serde_json::Value = serde_json::from_str(&stdout(&run(&["export", "functions", "--format", "json"]))).unwrap();
    assert!(j.as_array().unwrap().len() > 380);
    let csv = stdout(&run(&["export", "measures", "--format", "csv"]));
    assert!(csv.starts_with("id,name,kind,category"));
    let path = std::env::temp_dir().join("dax-glossary-test-export.csv");
    let o = run(&["export", "functions", "--format", "csv", "--output", path.to_str().unwrap()]);
    assert!(o.status.success());
    assert!(std::fs::read_to_string(&path).unwrap().starts_with("name,category"));
    let _ = std::fs::remove_file(path);
    assert!(!run(&["export", "functions", "--format", "xml"]).status.success());
}

#[test]
fn stats_validate_help_and_unknown_command() {
    assert!(stdout(&run(&["stats"])).contains("functions:"));
    let o = run(&["validate"]);
    assert!(o.status.success() && stdout(&o).starts_with("ok:"));
    assert!(stdout(&run(&["help"])).contains("USAGE"));
    assert_eq!(run(&["frobnicate"]).status.code(), Some(2));
}
