//! A simple INI-like parser for Klipper's `printer.cfg` format.

use crate::errors::MigrationError;
use std::collections::HashMap;

pub type ParsedConfig = HashMap<String, HashMap<String, String>>;

/// Parses the raw text content of a `printer.cfg` file.
///
/// It handles sections `[section_name]`, key-value pairs `key = value`,
/// and comments starting with `#` or `;`. Whitespace is trimmed.
///
/// # Returns
/// A nested HashMap where the outer keys are section names and the inner
/// HashMap contains the key-value pairs for that section.
pub fn parse_ini(content: &str) -> Result<ParsedConfig, MigrationError> {
    let mut config = ParsedConfig::new();
    let mut current_section_name: Option<String> = None;

    for (line_num, line) in content.lines().enumerate() {
        let line = line.trim();

        // Ignore comments and empty lines
        if line.is_empty() || line.starts_with('#') || line.starts_with(';') {
            continue;
        }

        // Section header
        if line.starts_with('[') && line.ends_with(']') {
            let section_name = line.trim_matches(|p| p == '[' || p == ']').to_string();
            config.insert(section_name.clone(), HashMap::new());
            current_section_name = Some(section_name);
            continue;
        }

        if let Some(section_name) = &current_section_name {
            let split_pos = line.find('=').or_else(|| line.find(':'));
            if let Some(pos) = split_pos {
                let key = line[..pos].trim().to_string();
                let value = line[pos+1..].trim().to_string();
                if let Some(section) = config.get_mut(section_name) {
                    section.insert(key, value);
                }
            } else {
                return Err(MigrationError::ParseError(
                    line_num + 1,
                    format!("Invalid key-value pair: {}", line),
                ));
            }
        }
    }

    Ok(config)
}
