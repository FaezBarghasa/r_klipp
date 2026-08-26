use anyhow::{anyhow, Result};
use std::collections::HashMap;

pub type IniSection = HashMap<String, String>;
pub type IniConfig = HashMap<String, IniSection>;

/// Parse INI file strictly preserving case sensitivity (`optionxform = str`).
/// Handles Python Moonraker comment conventions:
/// - `#` always starts a comment
/// - `;` starts a comment only if preceded by whitespace or at the start of a line.
pub fn parse_case_sensitive_ini(content: &str) -> Result<IniConfig> {
    let mut config: IniConfig = HashMap::new();
    let mut current_section_name: Option<String> = None;
    let mut current_section: IniSection = HashMap::new();

    for (line_num, raw_line) in content.lines().enumerate() {
        let stripped = raw_line.trim_start();
        if stripped.is_empty() || stripped.starts_with('#') || stripped.starts_with(';') {
            continue;
        }

        // Check for section header
        if stripped.starts_with('[') {
            if let Some(end_idx) = stripped.find(']') {
                if let Some(sec_name) = current_section_name.take() {
                    config.insert(sec_name, std::mem::take(&mut current_section));
                }
                let sec_name = stripped[1..end_idx].trim().to_string();
                current_section_name = Some(sec_name);
                continue;
            } else {
                return Err(anyhow!(
                    "Line {}: Invalid section header (missing closing bracket)",
                    line_num + 1
                ));
            }
        }

        // Strip inline comments
        let line_without_comment = strip_inline_comments(raw_line);
        let trimmed = line_without_comment.trim();
        if trimmed.is_empty() {
            continue;
        }

        // Parse key = value or key: value
        if let Some(pos) = trimmed.find('=').or_else(|| trimmed.find(':')) {
            let key = trimmed[..pos].trim().to_string();
            let value = trimmed[pos + 1..].trim().to_string();

            if let Some(_) = &current_section_name {
                current_section.insert(key, value);
            } else {
                return Err(anyhow!(
                    "Line {}: Key-value pair '{}' found outside any section",
                    line_num + 1,
                    key
                ));
            }
        }
    }

    if let Some(sec_name) = current_section_name {
        config.insert(sec_name, current_section);
    }

    Ok(config)
}

fn strip_inline_comments(line: &str) -> &str {
    let bytes = line.as_bytes();
    for i in 0..bytes.len() {
        if bytes[i] == b'#' {
            return &line[..i];
        }
        if bytes[i] == b';' && (i == 0 || bytes[i - 1].is_ascii_whitespace()) {
            return &line[..i];
        }
    }
    line
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_case_sensitive_ini_parsing() {
        let conf = r#"
[server]
host: 0.0.0.0
port: 7125
enable_debug_logging: True # standard comment
CaseSensitiveKey: ValueWithPreservedCase ; comment after whitespace

[file_manager]
enable_object_processing: False
"#;

        let parsed = parse_case_sensitive_ini(conf).unwrap();
        assert!(parsed.contains_key("server"));
        let server = &parsed["server"];
        assert_eq!(server.get("host").unwrap(), "0.0.0.0");
        assert_eq!(server.get("port").unwrap(), "7125");
        assert_eq!(server.get("enable_debug_logging").unwrap(), "True");
        assert_eq!(server.get("CaseSensitiveKey").unwrap(), "ValueWithPreservedCase");

        let fm = &parsed["file_manager"];
        assert_eq!(fm.get("enable_object_processing").unwrap(), "False");
    }
}
