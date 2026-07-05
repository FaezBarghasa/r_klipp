use tokio::io::{AsyncBufReadExt, BufReader};
use std::collections::HashMap;

pub struct GcodeParser {
    state: HashMap<char, f32>,
}

impl GcodeParser {
    pub fn new() -> Self {
        Self {
            state: HashMap::new(),
        }
    }

    pub async fn parse_line(&mut self, line: &str) {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let command = parts[0];
        match command {
            "G0" | "G1" => {
                for part in &parts[1..] {
                    if let Some(axis) = part.chars().next() {
                        if let Ok(val) = part[1..].parse::<f32>() {
                            self.state.insert(axis, val);
                        }
                    }
                }
            }
            "G28" => {
                // Home axes
            }
            "M104" | "M109" => {
                // Set temperature
            }
            _ => {
                // Handle other commands or macros
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_gcode_parser() {
        let mut parser = GcodeParser::new();
        let gcode = "G1 X10 Y20.5 Z0.4 F3000";
        parser.parse_line(gcode).await;

        assert_eq!(parser.state.get(&'X'), Some(&10.0));
        assert_eq!(parser.state.get(&'Y'), Some(&20.5));
        assert_eq!(parser.state.get(&'Z'), Some(&0.4));
        assert_eq!(parser.state.get(&'F'), Some(&3000.0));
    }
}