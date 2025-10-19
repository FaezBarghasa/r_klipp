//! Typed structs representing a validated and migrated Klipper configuration.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level configuration for the entire printer.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct PrinterConfig {
    pub kinematics: String,
    pub max_velocity: f32,
    pub max_accel: f32,
    pub steppers: StepperAxes,
    pub extruder: Option<HeaterConfig>,
    pub heater_bed: Option<HeaterConfig>,
    /// Stores any other sections that are not explicitly parsed.
    pub other_sections: HashMap<String, HashMap<String, String>>,
}

/// Container for the primary motion axes.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct StepperAxes {
    pub x: Option<AxisConfig>,
    pub y: Option<AxisConfig>,
    pub z: Option<AxisConfig>,
}

/// Configuration for a single stepper motor axis.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AxisConfig {
    pub stepper: StepperConfig,
    pub position_endstop: f32,
    pub position_max: f32,
    pub homing_speed: f32,
}

/// Low-level details for a stepper motor.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StepperConfig {
    pub step_pin: String,
    pub dir_pin: String,
    pub enable_pin: String,
    pub microsteps: u16,
    pub rotation_distance: f32,
}

/// Configuration for a heater element, like an extruder or a heated bed.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HeaterConfig {
    pub heater_pin: String,
    pub sensor_type: String,
    pub sensor_pin: String,
    pub min_temp: f32,
    pub max_temp: f32,
    // Extruder-specific fields, optional for other heaters
    pub stepper: Option<StepperConfig>,
    pub nozzle_diameter: Option<f32>,
    pub filament_diameter: Option<f32>,
}
