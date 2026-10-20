use std::collections::BTreeMap;
use std::process::ExitCode;

use dax_glossary::{catalog, Function, Measure};

const HELP: &str = "dax-glossary - every DAX function and ready-to-render DAX measure templates

USAGE:
  dax-glossary list functions|measures [--category <name>] [--tag <tag>] [--json]
  dax-glossary categories functions|measures
  dax-glossary search <text...> [--json]
  dax-glossary get function|measure <name|id> [--json]
  dax-glossary render <template-id> --param key=value [--param key=value ...] [--json]
  dax-glossary export functions|measures --format json|csv [--output <path>]
  dax-glossary stats
  dax-glossary validate
";

struct Opts {
    positional: Vec<String>,
    category: Option<String>,
    tag: Option<String>,
    json: bool,
    format: Option<String>,
    output: Option<String>,
    params: Vec<String>,
}

fn parse(args: &[String]) -> Result<Opts, String> {
    let mut o = Opts {
        positional: vec![],
        category: None,
        tag: None,
        json: false,
        format: None,
        output: None,
        params: vec![],
    };
    let mut it = args.iter();
    while let Some(a) = it.next() {
        let mut value = |flag: &str| it.next().cloned().ok_or_else(|| format!("{flag} needs a value"));
        match a.as_str() {
            "--category" => o.category = Some(value("--category")?),
            "--tag" => o.tag = Some(value("--tag")?),
            "--format" => o.format = Some(value("--format")?),
            "--output" => o.output = Some(value("--output")?),
            "--param" => o.params.push(value("--param")?),
            "--json" => o.json = true,
            s if s.starts_with("--") => return Err(format!("unknown option {s}")),
            _ => o.positional.push(a.clone()),
        }
    }
    Ok(o)
}

fn print_function(f: &Function) {
    println!("{}  [{}]  returns {}", f.name, f.category, f.returns);
    println!("  {}", f.syntax);
    for alt in &f.syntax_alternatives {
        println!("  {alt}");
    }
    println!("  {}", f.description);
    if !f.tags.is_empty() {
        println!("  tags: {}", f.tags.join(", "));
    }
    println!("  {}", f.reference_url);
}

fn print_measure(m: &Measure) {
    println!("{}  [{}]  {}", m.id, m.category, m.name);
    println!("  {}", m.description);
    for p in &m.parameters {
        println!(
            "  --param {}=<{}>  {} (e.g. {})",
            p.name,
            format!("{:?}", p.kind).to_lowercase(),
            p.description,
            p.example
        );
    }
    if !m.assumptions.is_empty() {
        println!("  assumes: {}", m.assumptions);
    }
}

fn write_or_print(text: String, output: Option<&String>) -> Result<(), String> {
    match output {
        Some(p) => std::fs::write(p, text).map_err(|e| format!("cannot write {p}: {e}")),
        None => {
            print!("{text}");
            Ok(())
        }
    }
}

fn run(args: &[String]) -> Result<(), String> {
    let c = catalog();
    let Some(cmd) = args.first() else {
        print!("{HELP}");
        return Ok(());
    };
    let o = parse(&args[1..])?;
    let what = o.positional.first().map(String::as_str);
    match cmd.as_str() {
        "list" => match what {
            Some("functions") => {
                let mut v: Vec<&Function> = match &o.category {
                    Some(cat) => c.functions_in(cat),
                    None => c.functions().iter().collect(),
                };
                if let Some(t) = &o.tag {
                    v.retain(|f| f.tags.iter().any(|x| x.eq_ignore_ascii_case(t)));
                }
                if o.json {
                    println!("{}", serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?);
                } else {
                    for f in v {
                        println!("{:<28} {:<20} {}", f.name, f.category, f.syntax);
                    }
                }
                Ok(())
            }
            Some("measures") => {
                let v: Vec<&Measure> = match &o.category {
                    Some(cat) => c.measures_in(cat),
                    None => c.measures().iter().collect(),
                };
                if o.json {
                    println!("{}", serde_json::to_string_pretty(&v).map_err(|e| e.to_string())?);
                } else {
                    for m in v {
                        println!("{:<26} {:<18} {}", m.id, m.category, m.description);
                    }
                }
                Ok(())
            }
            _ => Err("list needs 'functions' or 'measures'".into()),
        },
        "categories" => {
            let cats = match what {
                Some("functions") => c.function_categories(),
                Some("measures") => c.measure_categories(),
                _ => return Err("categories needs 'functions' or 'measures'".into()),
            };
            for cat in cats {
                println!("{cat}");
            }
            Ok(())
        }
        "search" => {
            let q = o.positional.join(" ");
            let hits = c.search(&q);
            if o.json {
                println!("{}", serde_json::to_string_pretty(&hits).map_err(|e| e.to_string())?);
            } else {
                for h in &hits {
                    match (h.function, h.measure) {
                        (Some(f), _) => println!("function {:<28} {}", f.name, f.description),
                        (_, Some(m)) => println!("measure  {:<28} {}", m.id, m.description),
                        _ => {}
                    }
                }
                if hits.is_empty() {
                    return Err(format!("nothing found for '{q}'"));
                }
            }
            Ok(())
        }
        "get" => {
            let name = o.positional.get(1).ok_or("get needs a name")?;
            match what {
                Some("function") => {
                    let f = c.get_function(name).map_err(|e| e.to_string())?;
                    if o.json {
                        println!("{}", serde_json::to_string_pretty(f).map_err(|e| e.to_string())?);
                    } else {
                        print_function(f);
                    }
                    Ok(())
                }
                Some("measure") => {
                    let m = c.get_measure(name).map_err(|e| e.to_string())?;
                    if o.json {
                        println!("{}", serde_json::to_string_pretty(m).map_err(|e| e.to_string())?);
                    } else {
                        print_measure(m);
                    }
                    Ok(())
                }
                _ => Err("get needs 'function' or 'measure'".into()),
            }
        }
        "render" => {
            let id = what.ok_or("render needs a template id")?;
            let mut p = BTreeMap::new();
            for kv in &o.params {
                let (k, v) = kv.split_once('=').ok_or_else(|| format!("--param '{kv}' must be key=value"))?;
                if p.insert(k.trim().to_string(), v.to_string()).is_some() {
                    return Err(format!("parameter '{}' given twice", k.trim()));
                }
            }
            let dax = c.render_measure(id, &p).map_err(|e| e.to_string())?;
            if o.json {
                let out = serde_json::json!({ "template": id, "dax": dax });
                println!("{}", serde_json::to_string_pretty(&out).map_err(|e| e.to_string())?);
            } else {
                println!("{dax}");
            }
            Ok(())
        }
        "export" => {
            let fmt = o.format.as_deref().unwrap_or("json");
            let text = match (what, fmt) {
                (Some("functions"), "json") => c.functions_json(),
                (Some("functions"), "csv") => c.functions_csv(),
                (Some("measures"), "json") => c.measures_json(),
                (Some("measures"), "csv") => c.measures_csv(),
                (Some("functions" | "measures"), other) => return Err(format!("unknown format '{other}' (json or csv)")),
                _ => return Err("export needs 'functions' or 'measures'".into()),
            };
            write_or_print(text, o.output.as_ref())
        }
        "stats" => {
            let cov = c.coverage();
            println!(
                "functions: {}   measure templates: {}   reviewed: {}",
                cov.function_count, cov.measure_template_count, cov.review_date
            );
            for (cat, n) in &cov.categories {
                println!("  {cat:<22} {n}");
            }
            println!("source: {}", cov.source);
            Ok(())
        }
        "validate" => {
            let issues = c.validate();
            if issues.is_empty() {
                println!("ok: {} functions, {} templates", c.functions().len(), c.measures().len());
                Ok(())
            } else {
                for i in &issues {
                    eprintln!("{i}");
                }
                Err(format!("{} integrity problem(s)", issues.len()))
            }
        }
        "help" | "--help" | "-h" => {
            print!("{HELP}");
            Ok(())
        }
        other => Err(format!("unknown command '{other}'\n\n{HELP}")),
    }
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match run(&args) {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}
