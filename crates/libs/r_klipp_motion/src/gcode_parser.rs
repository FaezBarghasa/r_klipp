use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::fs::File;
use std::path::Path;

pub struct GcodeParser {
    modal_g: u32,
    position: [f32; 3],
    extruder_pos: f32,
}

impl GcodeParser {
    pub fn new() -> Self {
        Self {
            modal_g: 0,
            position: [0.0, 0.0, 0.0],
            extruder_pos: 0.0,
        }
    }

    pub async fn parse_file(&mut self, path: &Path) -> Result<Vec<String>, std::io::Error> {
        let file = File::open(path).await?;
        let reader = BufReader::new(file);
        let mut lines = reader.lines();
        let mut commands = Vec::new();

        while let Some(line) = lines.next_line().await? {
            commands.push(self.parse_line(&line));
        }

        Ok(commands)
    }

    fn parse_line(&mut self, line: &str) -> String {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.is_empty() {
            return "".to_string();
        }

        let mut command = "".to_string();
        let mut g_code = self.modal_g;

        for part in parts {
            if part.starts_with(';') {
                break;
            }
            let code = &part[0..1];
            let value = &part[1..].parse::<f32>().unwrap_or(0.0);

            match code {
                "G" => {
                    g_code = *value as u32;
                    self.modal_g = g_code;
                    command = format!("{} G{}", command, g_code);
                }
                "X" => {
                    self.position[0] = *value;
                    command = format!("{} X{}", command, value);
                }
                "Y" => {
                    self.position[1] = *value;
                    command = format!("{} Y{}", command, value);
                }
                "Z" => {
                    self.position[2] = *value;
                    command = format!("{} Z{}", command, value);
                }
                "E" => {
                    self.extruder_pos = *value;
                    command = format!("{} E{}", command, value);
                }
                "M" => {
                    command = format!("{} M{}", command, value);
                }
                _ => {}
            }
        }
        command.trim().to_string()
    }

    pub fn get_position(&self) -> [f32; 3] {
        self.position
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;
    use tokio::io::AsyncWriteExt;

    #[tokio::test]
    async fn test_gcode_parser() {
        let file_path = Path::new("/tmp/test.gcode");
        let mut file = tokio::fs::File::create(&file_path).await.unwrap();
        file.write_all(b"G0 X10 Y20\nG1 Z5 E1.0\nX15\n").await.unwrap();

        let mut parser = GcodeParser::new();
        let commands = parser.parse_file(&file_path).await.unwrap();

        assert_eq!(commands.len(), 3);
        assert_eq!(commands[0], "G0 X10 Y20");
        assert_eq!(commands[1], "G1 Z5 E1");
        assert_eq!(parser.get_position(), [15.0, 20.0, 5.0]);
    }
}
