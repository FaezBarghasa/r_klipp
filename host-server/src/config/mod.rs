//! Configuration engine with case-sensitive INI parsing and Jinja2 templating.

pub mod jinja;
pub mod parser;

pub use jinja::TemplateEngine;
pub use parser::{parse_case_sensitive_ini, IniConfig};
