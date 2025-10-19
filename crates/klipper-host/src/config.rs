//! Klipper Configuration Loader
//!
//! This module is responsible for parsing and interpreting Klipper's `printer.cfg`
//! file format using an INI-style parser. It provides strongly-typed structs
//! for accessing configuration values.

use anyhow::{Context, Result};
use configparser::ini::Ini;
use std::path::Path;

/// Represents the [mcu] section of the config.
#[derive(Debug, Clone)]
pub struct McuConfig {
    pub serial_port: String,
    pub baud_rate: u32,
}

/// Represents a generic [stepper] section.
#[derive(Debug, Clone)]
pub struct StepperConfig {
    pub steps_per_mm: f32,
    pub max_velocity: f32,
}

/// Represents the main [printer] section.
#[derive(Debug, Clone)]
pub struct PrinterInfoConfig {
    pub kinematics: String,
    pub max_velocity: u32,
    pub max_accel: u32,
}

/// Holds the entire parsed printer configuration.
#[derive(Debug, Clone)]
pub struct PrinterConfig {
    pub mcu: McuConfig,
    pub printer: PrinterInfoConfig,
    pub stepper_x: StepperConfig,
    pub stepper_y: StepperConfig,
    pub stepper_z: StepperConfig,
}

impl PrinterConfig {
    /// Loads and parses the configuration file from the given path.
    pub fn load(path: &Path) -> Result<Self> {
        let mut config = Ini::new();
        config
            .load(path)
            .with_context(|| format!("Failed to load configuration file: {:?}", path))?;

        let mcu = McuConfig {
            serial_port: config.get("mcu", "serial").context("[mcu] serial not found")?,
            baud_rate: config.getuint("mcu", "baud").context("[mcu] baud not found")? as u32,
        };

        let printer = PrinterInfoConfig {
            kinematics: config.get("printer", "kinematics").context("[printer] kinematics not found")?,
            max_velocity: config.getuint("printer", "max_velocity").context("[printer] max_velocity not found")? as u32,
            max_accel: config.getuint("printer", "max_accel").context("[printer] max_accel not found")? as u32,
        };

        let stepper_x = Self::load_stepper(&config, "stepper_x")?;
        let stepper_y = Self::load_stepper(&config, "stepper_y")?;
        let stepper_z = Self::load_stepper(&config, "stepper_z")?;

        Ok(Self {
            mcu,
            printer,
            stepper_x,
            stepper_y,
            stepper_z,
        })
    }

    /// Helper function to load a stepper configuration section.
    fn load_stepper(config: &Ini, section: &str) -> Result<StepperConfig> {
        Ok(StepperConfig {
            steps_per_mm: config.getfloat(section, "steps_per_mm").with_context(|| format!("[{}] steps_per_mm not found", section))? as f32,
            max_velocity: config.getfloat(section, "max_velocity").with_context(|| format!("[{}] max_velocity not found", section))? as f32,
        })
    }
}

