use tokio::io::{AsyncBufReadExt, BufReader};
use std::collections::HashMap;

#[derive(Default)]
pub struct GcodeParser {
    modal_state: HashMap<char, f32>,
}

impl GcodeParser {
    pub fn new() -> Self {
        Default::default()
    }

    pub async fn parse_stream<R: tokio::io::AsyncRead + Unpin>(&mut self, reader: R) -> Vec<String> {
        let mut reader = BufReader::new(reader);
        let mut lines = Vec::new();
        let mut line = String::new();

        while reader.read_line(&mut line).await.unwrap() > 0 {
            self.parse_line(&line);
            lines.push(line.clone());
            line.clear();
        }
        lines
    }

    fn parse_line(&mut self, line: &str) {
        let line = line.split(';').next().unwrap_or("").trim();
        if line.is_empty() {
            return;
        }

        let mut parts = line.split_whitespace();
        let command = parts.next().unwrap_or("");

        match command {
            "G0" | "G1" => {
                for part in parts {
                    let key = part.chars().next().unwrap();
                    let value = part[1..].parse::<f32>().unwrap();
                    self.modal_state.insert(key, value);
                }
            }
            "M104" | "M109" => {
                // Handle temp commands
            }
            _ => {
                // Handle macros or other commands
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
        let gcode = "G1 X10 Y20.5 F3000 ; comment\nG0 Z5".as_bytes();
        parser.parse_stream(gcode).await;

        assert_eq!(parser.modal_state.get(&'X'), Some(&10.0));
        assert_eq!(parser.modal_state.get(&'Y'), Some(&20.5));
        assert_eq!(parser.modal_state.get(&'F'), Some(&3000.0));
        assert_eq!(parser.modal_state.get(&'Z'), Some(&5.0));
    }
}
