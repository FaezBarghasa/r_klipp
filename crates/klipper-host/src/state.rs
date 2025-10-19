//! Printer State Management
//!
//! This module defines the central `PrinterState` struct, which holds all dynamic
//! information about the printer, such as its position, temperatures, and connection status.
//! It is designed to be safely shared across multiple concurrent tasks.

use serde::Serialize;
use std::collections::HashMap;

/// Represents the current position of the toolhead.
#[derive(Debug, Clone, Serialize)]
pub struct Position {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub e: f32,
}

impl Default for Position {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            e: 0.0,
        }
    }
}

/// Represents the temperature of a sensor.
#[derive(Debug, Clone, Serialize)]
pub struct Temperature {
    pub actual: f32,
    pub target: f32,
}

impl Default for Temperature {
    fn default() -> Self {
        Self {
            actual: 0.0,
            target: 0.0,
        }
    }
}

/// The overall state of the printer.
#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq)]
pub enum PrinterStatus {
    Initializing,
    Ready,
    Printing,
    Error,
    Disconnected,
}

/// A thread-safe container for all dynamic printer state.
#[derive(Debug, Clone, Serialize)]
pub struct PrinterState {
    pub status: PrinterStatus,
    pub status_message: String,
    pub position: Position,
    pub temperatures: HashMap<String, Temperature>,
}

impl PrinterState {
    /// Creates a new `PrinterState` with default values.
    pub fn new() -> Self {
        let mut temperatures = HashMap::new();
        temperatures.insert(
            "extruder".to_string(),
            Temperature {
                actual: 21.0,
                target: 0.0,
            },
        );
        temperatures.insert(
            "heater_bed".to_string(),
            Temperature {
                actual: 22.0,
                target: 0.0,
            },
        );

        Self {
            status: PrinterStatus::Initializing,
            status_message: "Server is starting...".to_string(),
            position: Position::default(),
            temperatures,
        }
    }
}

impl Default for PrinterState {
    fn default() -> Self {
        Self::new()
    }
}
