use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::catalog::{kind_str, Catalog, Error, MeasureKind, ParamKind};

#[derive(Debug, PartialEq)]
pub enum RenderError {
    UnknownTemplate(String),
    MissingParameters(Vec<String>),
    UnexpectedParameters(Vec<String>),
    InvalidValue {
        parameter: String,
        kind: &'static str,
        reason: String,
    },
    /// The template body references a placeholder that is not declared (a catalog bug).
    UndeclaredPlaceholder(String),
}

impl fmt::Display for RenderError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RenderError::UnknownTemplate(t) => write!(f, "unknown measure template '{t}'"),
            RenderError::MissingParameters(p) => write!(f, "missing parameter(s): {}", p.join(", ")),
            RenderError::UnexpectedParameters(p) => write!(f, "unexpected parameter(s): {}", p.join(", ")),
            RenderError::InvalidValue { parameter, kind, reason } => write!(f, "parameter '{parameter}' is not a valid {kind}: {reason}"),
            RenderError::UndeclaredPlaceholder(p) => write!(f, "template uses undeclared placeholder {{{{{p}}}}}"),
        }
    }
}
impl std::error::Error for RenderError {}

impl From<Error> for RenderError {
    fn from(e: Error) -> Self {
        match e {
            Error::UnknownMeasure(id) => RenderError::UnknownTemplate(id),
            other => RenderError::UnknownTemplate(other.to_string()),
        }
    }
}

/// Placeholder names (`{{name}}`) of a template body, in order of appearance.
pub(crate) fn placeholders(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = body;
    while let Some(i) = rest.find("{{") {
        let after = &rest[i + 2..];
        match after.find("}}") {
            Some(j) => {
                out.push(after[..j].trim().to_string());
                rest = &after[j + 2..];
            }
            None => break,
        }
    }
    out
}

impl Catalog {
    /// Renders a template into DAX. Measures become `Name = <body>` (a multi-line body starts on the next line);
    /// calculation items render as the bare expression. Every declared parameter must be given, and no other.
    /// The result is only as correct as the model it is pasted into (table, column and relationship names are yours).
    pub fn render_measure(&self, id: &str, params: &BTreeMap<String, String>) -> Result<String, RenderError> {
        let m = self.get_measure(id)?;
        let declared: BTreeSet<&str> = m.parameters.iter().map(|p| p.name.as_str()).collect();
        let missing: Vec<String> = m
            .parameters
            .iter()
            .filter(|p| !params.contains_key(&p.name))
            .map(|p| p.name.clone())
            .collect();
        if !missing.is_empty() {
            return Err(RenderError::MissingParameters(missing));
        }
        let extra: Vec<String> = params.keys().filter(|k| !declared.contains(k.as_str())).cloned().collect();
        if !extra.is_empty() {
            return Err(RenderError::UnexpectedParameters(extra));
        }
        let mut rendered: BTreeMap<&str, String> = BTreeMap::new();
        for p in &m.parameters {
            rendered.insert(p.name.as_str(), render_value(&p.name, p.kind, &params[&p.name])?);
        }
        // one pass over the body: substituted values are never scanned again
        let mut body = String::new();
        let mut rest = m.body.as_str();
        while let Some(i) = rest.find("{{") {
            body.push_str(&rest[..i]);
            let after = &rest[i + 2..];
            let Some(j) = after.find("}}") else {
                body.push_str("{{");
                rest = after;
                continue;
            };
            let name = after[..j].trim();
            match rendered.get(name) {
                Some(v) => body.push_str(v),
                None => return Err(RenderError::UndeclaredPlaceholder(name.to_string())),
            }
            rest = &after[j + 2..];
        }
        body.push_str(rest);
        Ok(match m.kind {
            MeasureKind::CalculationItem => body,
            MeasureKind::Measure => {
                let name = rendered.get("measure_name").cloned().unwrap_or_else(|| m.name.clone());
                if body.contains('\n') {
                    format!("{name} =\n{body}")
                } else {
                    format!("{name} = {body}")
                }
            }
        })
    }
}

fn bad(parameter: &str, kind: &'static str, reason: &str) -> RenderError {
    RenderError::InvalidValue {
        parameter: parameter.to_string(),
        kind,
        reason: reason.to_string(),
    }
}

fn render_value(name: &str, kind: ParamKind, raw: &str) -> Result<String, RenderError> {
    let v = raw.trim();
    if v.is_empty() {
        return Err(bad(name, kind_str(kind), "empty"));
    }
    if v.chars().any(|c| c == '\0' || (c.is_control() && c != '\n' && c != '\t')) {
        return Err(bad(name, kind_str(kind), "control characters"));
    }
    match kind {
        ParamKind::Table => table_ref(v)
            .and_then(|n| {
                if n == v.len() {
                    Ok(())
                } else {
                    Err("unexpected characters after the table name".to_string())
                }
            })
            .map(|_| v.to_string())
            .map_err(|r| bad(name, "table reference", &r)),
        ParamKind::Column => column_ref(v)
            .map(|_| v.to_string())
            .map_err(|r| bad(name, "column reference ('Table'[Column])", &r)),
        ParamKind::Measure => bracket_name(v)
            .map(|_| v.to_string())
            .map_err(|r| bad(name, "measure reference ([Name])", &r)),
        ParamKind::Number => v
            .parse::<f64>()
            .ok()
            .filter(|n| n.is_finite())
            .map(|_| v.to_string())
            .ok_or_else(|| bad(name, "number", "not a finite number")),
        ParamKind::Text => {
            if v.contains('\n') {
                return Err(bad(name, "text", "line breaks are not allowed"));
            }
            Ok(format!("\"{}\"", v.replace('"', "\"\"")))
        }
        ParamKind::Name => {
            if v.contains('\n') || v.contains('=') {
                return Err(bad(name, "measure name", "must not contain '=' or line breaks"));
            }
            Ok(v.to_string())
        }
        ParamKind::Expression => balanced(v).map(|_| v.to_string()).map_err(|r| bad(name, "DAX expression", &r)),
    }
}

/// `Table` or `'Table name'` (a quote inside is doubled); returns the number of bytes consumed.
fn table_ref(s: &str) -> Result<usize, String> {
    if let Some(rest) = s.strip_prefix('\'') {
        let mut chars = rest.char_indices().peekable();
        while let Some((i, c)) = chars.next() {
            if c == '\'' {
                if matches!(chars.peek(), Some((_, '\''))) {
                    chars.next();
                    continue;
                }
                return if i == 0 { Err("empty quoted name".into()) } else { Ok(i + 2) };
            }
        }
        Err("unterminated quote".into())
    } else {
        let n: usize = s
            .chars()
            .take_while(|c| c.is_alphanumeric() || *c == '_')
            .map(|c| c.len_utf8())
            .sum();
        if n == 0 || s.chars().next().is_some_and(|c| c.is_ascii_digit()) {
            return Err("use letters, digits and _ (starting with a letter) or quote the name".into());
        }
        Ok(n)
    }
}

fn bracket_name(s: &str) -> Result<(), String> {
    let inner = s.strip_prefix('[').ok_or("expected [")?;
    let inner = inner.strip_suffix(']').ok_or("missing closing ]")?;
    // a ] inside a name is written ]]
    if inner.trim().is_empty() || inner.replace("]]", "").contains(']') {
        return Err("empty or malformed name".into());
    }
    Ok(())
}

fn column_ref(s: &str) -> Result<(), String> {
    if s.starts_with('[') {
        return Err("a column needs its table: 'Table'[Column]".into());
    }
    let n = table_ref(s)?;
    bracket_name(&s[n..])
}

/// Parentheses/brackets/braces balanced, ignoring string literals ("...") and quoted names ('...').
fn balanced(s: &str) -> Result<(), String> {
    let mut stack = Vec::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        match c {
            '"' | '\'' => loop {
                match chars.next() {
                    Some(q) if q == c && chars.peek() == Some(&c) => {
                        chars.next();
                    }
                    Some(q) if q == c => break,
                    Some(_) => {}
                    None => return Err("unterminated string or quoted name".into()),
                }
            },
            '(' | '[' | '{' => stack.push(c),
            ')' | ']' | '}' => {
                let want = match c {
                    ')' => '(',
                    ']' => '[',
                    _ => '{',
                };
                if stack.pop() != Some(want) {
                    return Err(format!("unbalanced '{c}'"));
                }
            }
            _ => {}
        }
    }
    if stack.is_empty() {
        Ok(())
    } else {
        Err("unclosed bracket".into())
    }
}
