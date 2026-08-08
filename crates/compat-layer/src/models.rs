//! Typed structs representing a validated and migrated Klipper configuration.
// These models provide a structured, safe way to access printer settings after
// they have been parsed from a `printer.cfg` file.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level configuration for the entire printer.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct PrinterConfig {
    /// The type of kinematics (e.g., "cartesian", "corexy").
    pub kinematics: String,
    /// The maximum velocity of the toolhead in mm/s.
    pub max_velocity: f32,
    /// The maximum acceleration of the toolhead in mm/s^2.
    pub max_accel: f32,
    /// Configuration for the primary X, Y, and Z stepper axes.
    pub steppers: StepperAxes,
    /// Configuration for the primary extruder.
    pub extruder: Option<HeaterConfig>,
    /// Configuration for the heated bed.
    pub heater_bed: Option<HeaterConfig>,
    /// Stores any other sections from the config file that were not explicitly parsed.
    /// The outer key is the section name (e.g., "[fan]"), and the inner map
    /// contains the key-value pairs from that section.
    pub other_sections: HashMap<String, HashMap<String, String>>,
}

/// Container for the primary motion axes.
#[derive(Debug, Serialize, Deserialize, PartialEq, Default)]
pub struct StepperAxes {
    /// The X-axis stepper configuration.
    pub x: Option<AxisConfig>,
    /// The Y-axis stepper configuration.
    pub y: Option<AxisConfig>,
    /// The Z-axis stepper configuration.
    pub z: Option<AxisConfig>,
}

/// Configuration for a single stepper motor axis.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct AxisConfig {
    /// The low-level stepper motor configuration.
    pub stepper: StepperConfig,
    /// The position of the endstop for this axis.
    pub position_endstop: f32,
    /// The maximum travel distance for this axis.
    pub position_max: f32,
    /// The speed at which to home this axis.
    pub homing_speed: f32,
}

/// Low-level details for a stepper motor.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct StepperConfig {
    /// The MCU pin connected to the STEP input of the driver.
    pub step_pin: String,
    /// The MCU pin connected to the DIR input of the driver.
    pub dir_pin: String,
    /// The MCU pin connected to the ENABLE input of the driver.
    pub enable_pin: String,
    /// The number of microsteps per full step.
    pub microsteps: u16,
    /// The distance the axis travels for one full rotation of the motor.
    pub rotation_distance: f32,
}

/// Configuration for a heater element, like an extruder or a heated bed.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HeaterConfig {
    /// The MCU pin connected to the heater's MOSFET.
    pub heater_pin: String,
    /// The type of temperature sensor used (e.g., "EPCOS 100K B57560G104F").
    pub sensor_type: String,
    /// The MCU pin connected to the temperature sensor.
    pub sensor_pin: String,
    /// The minimum safe temperature for this heater.
    pub min_temp: f32,
    /// The maximum safe temperature for this heater.
    pub max_temp: f32,
    /// Extruder-specific fields. `None` for a heated bed.
    pub stepper: Option<StepperConfig>,
    /// The diameter of the nozzle. `None` for a heated bed.
    pub nozzle_diameter: Option<f32>,
    /// The diameter of the filament. `None` for a heated bed.
    pub filament_diameter: Option<f32>,
}
