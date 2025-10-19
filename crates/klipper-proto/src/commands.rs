#![deny(clippy::all)]
#![deny(warnings)]

//! Typed command and response structs & enums.
//!
//! These structures represent the data payload of Klipper messages.

/// A top-level message that can be either a command from the host
/// or a response from the MCU.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Message<'a> {
    /// A command sent from the host to the MCU.
    Command(Command<'a>),
    /// A response sent from the MCU to the host.
    Response(Response<'a>),
}

/// A command to queue a single step for one or more motors.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub struct CommandQueueStep {
    pub interval_ticks: u32,
    pub count: u16,
    pub add: i16,
}

/// Commands sent from the host computer to the MCU.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Command<'a> {
    Identify {
        dict_version: &'a [u8],
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
    // Add other commands as they are implemented...
    Unknown(u8, &'a [u8]),
}

/// Responses sent from the MCU back to the host computer.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum Response<'a> {
    Identify {
        is_config_valid: bool,
        version: &'a [u8],
        mcu_name: &'a [u8],
    },
    Status {
        // Define status fields as needed
    },
    Config {
        // Define config fields as needed
    },
    // Add other responses...
    Unknown,
}
