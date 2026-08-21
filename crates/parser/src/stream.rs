//! G-Code streaming and buffer handling for no_std environments.

use crate::lexer::{tokenize_line, Token};
use crate::ast::{parse_tokens, AstNode};
use crate::modal::ModalState;
use heapless::Vec;

pub struct GcodeLineParser {
    pub modal_state: ModalState,
}

impl GcodeLineParser {
    pub fn new() -> Self {
        Self {
            modal_state: ModalState::default(),
        }
    }

    /// Parses a single line string into an `AstNode` without heap allocation.
    pub fn parse_line<'a>(&mut self, line: &'a str) -> Result<Option<AstNode<'a>>, ()> {
        let tokens: Vec<Token<'a>, 32> = tokenize_line(line)?;
        parse_tokens(&tokens, &mut self.modal_state)
    }
}

impl Default for GcodeLineParser {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_gcode_lines() {
        let mut parser = GcodeLineParser::new();

        let node1 = parser.parse_line("G28").unwrap().unwrap();
        assert_eq!(node1, AstNode::Home { x: true, y: true, z: true });

        let node2 = parser.parse_line("G1 X100 Y50 E2 F1200").unwrap().unwrap();
        match node2 {
            AstNode::LinearMove { x, y, z, e, feedrate } => {
                assert_eq!(x, Some(100.0));
                assert_eq!(y, Some(50.0));
                assert_eq!(z, None);
                assert_eq!(e, Some(2.0));
                assert_eq!(feedrate, Some(1200.0));
            }
            _ => panic!("Expected linear move"),
        }

        let node3 = parser.parse_line("M104 S215").unwrap().unwrap();
        assert_eq!(node3, AstNode::SetHotendTemp { temp: 215.0, wait: false });
    }
}
