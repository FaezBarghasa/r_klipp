#![deny(clippy::all)]
#![deny(warnings)]

//! Typed command and response structs & enums.
//!
//! These structures represent the data payload of Klipper messages.

use serde::{Serialize, Deserialize};
use heapless::{Vec, String};

/// A top-level message that can be either a command from the host
/// or a response from the MCU.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Message {
    /// A command sent from the host to the MCU.
    Command(Command),
    /// A response sent from the MCU to the host.
    Response(Response),
}

/// A command to queue a single step for one or more motors.
#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub struct CommandQueueStep {
    pub interval_ticks: u32,
    pub count: u16,
    pub add: i16,
}

/// Commands sent from the host computer to the MCU.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Command {
    Identify {
        dict_version: Vec<u8, 32>,
    },
    GetConfig,
    GetStatus,
    QueueStep(CommandQueueStep),
    SetDigitalOut {
        pin: u8,
        value: u8,
    },
    SetPwmOut {
        pin: u8,
        value: u16,
    },
    GCode(String<64>),
    EmergencyStop,
    // Add other commands as they are implemented...
    Unknown(u8, Vec<u8, 32>),
}

/// Responses sent from the MCU back to the host computer.
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub enum Response {
    Identify {
        is_config_valid: bool,
        version: Vec<u8, 32>,
        mcu_name: Vec<u8, 32>,
    },
    Status {
        // Define status fields as needed
    },
    Config {
        is_config_valid: bool,
        mcu_version: u32,
        mcu_name: String<32>,
    },
    GCodeOk,
    GCodeError(String<64>),
    Log(String<64>),
    // Add other responses...
    Unknown,
}
