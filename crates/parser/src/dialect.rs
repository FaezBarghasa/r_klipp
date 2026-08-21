//! Machine Dialect Translation for G-Code commands.

use crate::ast::AstNode;
use crate::modal::ModalState;

#[derive(Debug, Clone, PartialEq)]
pub enum MachineCommand {
    LinearMove { x: Option<f32>, y: Option<f32>, z: Option<f32>, e: Option<f32>, f: Option<f32> },
    RapidMove { x: Option<f32>, y: Option<f32>, z: Option<f32> },
    Dwell(f32),
    Home { x: bool, y: bool, z: bool },
    SetTemperature { target_c: f32, heater_id: u8, wait: bool },
    SetFanSpeed(f32),
    EmergencyStop,
    NoOp,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    UnsupportedCommand,
    InvalidParameters,
}

pub trait MachineDialect {
    fn interpret(&self, node: &AstNode, state: &mut ModalState) -> Result<MachineCommand, ParseError>;
}

pub struct FdmPrinterDialect;

impl MachineDialect for FdmPrinterDialect {
    fn interpret(&self, node: &AstNode, _state: &mut ModalState) -> Result<MachineCommand, ParseError> {
        match node {
            AstNode::LinearMove { x, y, z, e, feedrate } => Ok(MachineCommand::LinearMove {
                x: *x,
                y: *y,
                z: *z,
                e: *e,
                f: *feedrate,
            }),
            AstNode::RapidMove { x, y, z, .. } => Ok(MachineCommand::RapidMove {
                x: *x,
                y: *y,
                z: *z,
            }),
            AstNode::Dwell(ms) => Ok(MachineCommand::Dwell(*ms)),
            AstNode::Home { x, y, z } => Ok(MachineCommand::Home { x: *x, y: *y, z: *z }),
            AstNode::SetHotendTemp { temp, wait } => Ok(MachineCommand::SetTemperature {
                target_c: *temp,
                heater_id: 0,
                wait: *wait,
            }),
            AstNode::SetBedTemp { temp, wait } => Ok(MachineCommand::SetTemperature {
                target_c: *temp,
                heater_id: 1,
                wait: *wait,
            }),
            AstNode::SetFanSpeed(speed) => Ok(MachineCommand::SetFanSpeed(*speed)),
            AstNode::FanOff => Ok(MachineCommand::SetFanSpeed(0.0)),
            AstNode::EmergencyStop => Ok(MachineCommand::EmergencyStop),
            AstNode::Comment(_) | AstNode::SetModalState(_) | AstNode::SetPosition { .. } => Ok(MachineCommand::NoOp),
            _ => Err(ParseError::UnsupportedCommand),
        }
    }
}
