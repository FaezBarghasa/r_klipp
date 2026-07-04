//! Asynchronous G-code streaming and lookahead buffer.
//! This corresponds to Task 2.4.

#![cfg_attr(not(test), no_std)]

use heapless::{VecDeque, String};
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};

use super::ast::{AstNode, build_ast_node};
use super::dialect::{MachineCommand, MachineDialect, MachineState, ParseError};
use super::lexer::{tokenize_line, Token};

/// A channel for sending raw G-code lines to the parser.
pub static GCODE_RX_CHANNEL: Channel<CriticalSectionRawMutex, String<128>, 4> = Channel::new();

/// The lookahead buffer for `MachineCommand`s.
pub static COMMAND_BUFFER: Channel<CriticalSectionRawMutex, MachineCommand, 1024> = Channel::new();

/// The main asynchronous parser task.
/// It reads raw G-code lines, parses them, interprets them using a dialect,
/// and pushes the resulting `MachineCommand`s into the lookahead buffer.
#[embassy_executor::task]
pub async fn gcode_parser_task(dialect: &'static dyn MachineDialect) {
    let mut machine_state = MachineState::default();

    loop {
        // Wait for a line of G-code from the input channel.
        let line = GCODE_RX_CHANNEL.receive().await;

        // Attempt to tokenize the line.
        match tokenize_line(&line) {
            Ok((_rem, tokens)) => {
                // If tokenization is successful, build the AST node.
                match build_ast_node(&tokens, &mut machine_state.modals) {
                    Ok(Some(ast_node)) => {
                        // Interpret the AST node using the provided machine dialect.
                        match dialect.interpret(&ast_node, &mut machine_state) {
                            Ok(Some(command)) => {
                                // Push the command to the lookahead buffer.
                                // This will block if the buffer is full, providing backpressure.
                                COMMAND_BUFFER.send(command).await;
                            }
                            Ok(None) => {
                                // The AST node was valid but didn't produce a machine command (e.g., a modal change).
                            }
                            Err(e) => {
                                // defmt::error!("G-code interpretation error: {:?}", e);
                            }
                        }
                    }
                    Ok(None) => {
                        // Line was empty or only contained comments/whitespace.
                    }
                    Err(e) => {
                        // defmt::error!("AST building error: {:?}", e);
                    }
                }
            }
            Err(e) => {
                // defmt::error!("G-code tokenization error: {:?}", e);
            }
        }
        // Yield to the executor to allow other tasks to run.
        Timer::after(Duration::from_micros(100)).await;
    }
}