//! Defines the modal state for G-code parsing.
//! This corresponds to part of Task 2.2.

#![cfg_attr(not(test), no_std)]

/// Represents the active G-code plane for arc moves.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ActivePlane {
    XY, // G17
    XZ, // G18
    YZ, // G19
}

/// Represents the unit system in use.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Units {
    Inches, // G20
    Millimeters, // G21
}

/// Represents the distance mode for axis movements.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DistanceMode {
    Absolute, // G90
    Relative, // G91
}

/// Represents the current coordinate system.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CoordinateSystem {
    G54,
    G55,
    G56,
    G57,
    G58,
    G59,
}

/// The `ModalState` struct holds the current state of the G-code interpreter.
/// These values are modified by certain G-codes and persist until changed.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ModalState {
    pub plane: ActivePlane,
    pub units: Units,
    pub distance_mode: DistanceMode,
    pub coordinate_system: CoordinateSystem,
    pub feed_rate: f32,
    pub spindle_speed: f32,
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            plane: ActivePlane::XY,
            units: Units::Millimeters,
            distance_mode: DistanceMode::Absolute,
            coordinate_system: CoordinateSystem::G54,
            feed_rate: 0.0,
            spindle_speed: 0.0,
        }
    }
}