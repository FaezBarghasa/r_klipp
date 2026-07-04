//! Defines the Abstract Syntax Tree (AST) for G-code commands.
//! This corresponds to part of Task 2.2.

#![cfg_attr(not(test), no_std)]

use heapless::Vec;
use super::modal::ModalState;

/// Represents a single command in the G-code language, parsed into a structured format.
#[derive(Debug, PartialEq, Clone, Copy)]
pub enum AstNode {
    /// G0 or G1: A linear movement.
    LinearMove {
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
        a: Option<f32>,
        b: Option<f32>,
        c: Option<f32>,
        e: Option<f32>,
        is_rapid: bool, // True for G0, false for G1
    },
    /// G2 or G3: An arc movement.
    ArcMove {
        x: Option<f32>,
        y: Option<f32>,
        z: Option<f32>,
        i: f32, // Arc center offset
        j: f32, // Arc center offset
        k: f32, // Arc center offset
        is_clockwise: bool, // True for G2, false for G3
    },
    /// G4: A dwell or pause.
    Dwell {
        milliseconds: u32,
    },
    /// M3, M4, M5: Spindle control.
    SpindleControl {
        speed: Option<f32>,
        is_clockwise: bool, // M3=true, M4=false
        is_on: bool, // M5=false
    },
    /// M2 or M30: Program end.
    ProgramEnd,
    /// T command: Tool change.
    ToolChange(u16),
    /// A command that only modifies the modal state.
    SetModalState(ModalState),
}

/// Parses a vector of tokens into a single `AstNode`.
/// This function is the core of the parser, applying the modal state to generate a complete command.
pub fn build_ast_node<'a>(tokens: &Vec<super::lexer::Token<'a>, 64>, state: &mut ModalState) -> Result<Option<AstNode>, &'static str> {
    // This is a placeholder implementation. A full implementation would be a state machine
    // that iterates through tokens, updates the modal state, and constructs the appropriate AstNode.
    // For now, we will just return Ok(None) to satisfy the compiler.
    if tokens.is_empty() {
        return Ok(None);
    }

    // A real implementation would be much more complex.
    // It would need to handle multiple G-codes on one line, update the modal state,
    // and then build the correct AstNode.

    // For demonstration, let's handle a simple G1 move.
    let mut g_code = None;
    let mut x = None;
    let mut y = None;
    let mut z = None;

    for token in tokens {
        match *token {
            super::lexer::Token::G(code) => g_code = Some(code),
            super::lexer::Token::Axis('X', val) => x = Some(val),
            super::lexer::Token::Axis('Y', val) => y = Some(val),
            super::lexer::Token::Axis('Z', val) => z = Some(val),
            super::lexer::Token::Feedrate(f) => state.feed_rate = f,
            _ => {} // Ignore other tokens for this simple example
        }
    }

    if let Some(1) = g_code {
        return Ok(Some(AstNode::LinearMove {
            x, y, z, a: None, b: None, c: None, e: None, is_rapid: false
        }));
    }

    Ok(None)
}
