//! Defines the machine dialect trait system for G-code interpretation.
//! This corresponds to Task 2.3.

#![cfg_attr(not(test), no_std)]

use super::ast::AstNode;
use super::modal::ModalState;
use heapless::Vec;

/// Represents a command fully interpreted for a specific machine.
#[derive(Debug, PartialEq)]
pub enum MachineCommand {
    LinearMove(Vec<f32, 8>), // Target positions for each axis
    Dwell(u32), // Milliseconds
    SpindleOn(f32),
    SpindleOff,
    ToolChange(u16),
    // Add other machine-specific commands here
}

/// A generic error for parsing or interpretation failures.
#[derive(Debug, PartialEq)]
pub enum ParseError {
    InvalidGCode,
    UnsupportedCommand,
    KinematicsError,
}

/// The `MachineState` holds the complete state of the machine, including all axis positions.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MachineState {
    pub modals: ModalState,
    pub axis_positions: [f32; 8], // X, Y, Z, A, B, C, E, U
}

impl Default for MachineState {
    fn default() -> Self {
        Self {
            modals: ModalState::default(),
            axis_positions: [0.0; 8],
        }
    }
}

/// The `MachineDialect` trait allows for different interpretations of the G-code AST.
/// This enables support for various machine types like FDM printers, CNC mills, etc.
pub trait MachineDialect {
    fn interpret(&self, node: &AstNode, state: &mut MachineState) -> Result<Option<MachineCommand>, ParseError>;
}

/// A dialect for standard 3-axis CNC routers.
pub struct CncRouterDialect;

impl MachineDialect for CncRouterDialect {
    fn interpret(&self, node: &AstNode, state: &mut MachineState) -> Result<Option<MachineCommand>, ParseError> {
        match node {
            AstNode::LinearMove { x, y, z, .. } => {
                let mut target = state.axis_positions;
                if let Some(x_pos) = x { target[0] = *x_pos; }
                if let Some(y_pos) = y { target[1] = *y_pos; }
                if let Some(z_pos) = z { target[2] = *z_pos; }

                let mut target_vec = Vec::new();
                target_vec.extend_from_slice(&target).unwrap();

                state.axis_positions = target;
                Ok(Some(MachineCommand::LinearMove(target_vec)))
            }
            AstNode::Dwell { milliseconds } => Ok(Some(MachineCommand::Dwell(*milliseconds))),
            _ => Ok(None), // Other commands not handled by this simple dialect
        }
    }
}

/// A dialect for FDM 3D printers.
pub struct FdmPrinterDialect;

impl MachineDialect for FdmPrinterDialect {
    fn interpret(&self, node: &AstNode, state: &mut MachineState) -> Result<Option<MachineCommand>, ParseError> {
        // FDM-specific interpretation would go here.
        // For example, handling extruder (E-axis) moves, retractions, etc.
        Ok(None)
    }
}

/// A dialect for Wire EDM machines.
pub struct WireEdmDialect;

impl MachineDialect for WireEdmDialect {
    fn interpret(&self, node: &AstNode, state: &mut MachineState) -> Result<Option<MachineCommand>, ParseError> {
        // Wire EDM-specific interpretation, like handling U/V axes for taper cuts.
        Ok(None)
    }
}