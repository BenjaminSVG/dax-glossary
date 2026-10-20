//! # dax-glossary
//!
//! A compile-time embedded catalog of **every DAX function** (syntax, arguments, return kind, category, reference link)
//! plus **measure templates** that render into ready-to-use DAX.
//!
//! ```
//! use dax_glossary::catalog;
//! use std::collections::BTreeMap;
//!
//! let c = catalog();
//! assert_eq!(c.get_function("totalytd").unwrap().name, "TOTALYTD");
//!
//! let mut p = BTreeMap::new();
//! p.insert("measure_name".to_string(), "Sales YTD".to_string());
//! p.insert("measure".to_string(), "[Sales]".to_string());
//! p.insert("dates".to_string(), "'Date'[Date]".to_string());
//! assert_eq!(c.render_measure("ytd", &p).unwrap(), "Sales YTD = TOTALYTD([Sales], 'Date'[Date])");
//! ```
mod catalog;
mod render;

pub use catalog::{catalog, Arg, Catalog, Coverage, Error, Function, Measure, MeasureKind, ParamKind, Parameter, SearchHit};
pub use render::RenderError;
