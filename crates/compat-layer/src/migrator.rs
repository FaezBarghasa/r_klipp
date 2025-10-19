//! Core logic for migrating parsed configuration data into typed structs.

use crate::errors::MigrationError;
use crate::models::*;
use crate::parser::{parse_ini, ParsedConfig};
use std::collections::HashMap;

/// A report containing warnings or other notes from the migration process.
#[derive(Debug, Default)]
pub struct MigrationReport {
    pub warnings: Vec<String>,
    pub unsupported_sections: Vec<String>,
}

/// Migrates a raw `printer.cfg` string into a `PrinterConfig` and a `MigrationReport`.
///
/// This is the main entry point for the library.
pub fn migrate_config(content: &str) -> Result<(PrinterConfig, MigrationReport), MigrationError> {
    let parsed_config = parse_ini(content)?;
    let mut report = MigrationReport::default();
    let config = map_to_structs(&parsed_config, &mut report)?;
    Ok((config, report))
}

/// Maps the parsed key-value data into the strongly-typed `PrinterConfig`.
fn map_to_structs(
    parsed: &ParsedConfig,
    report: &mut MigrationReport,
) -> Result<PrinterConfig, MigrationError> {
    let mut config = PrinterConfig::default();

    // --- Printer Section (Required) ---
    let printer_section = parsed
        .get("printer")
        .ok_or_else(|| MigrationError::MissingSection("printer".to_string()))?;

    config.kinematics = get_required(printer_section, "kinematics", "printer")?;
    config.max_velocity = get_parsed(printer_section, "max_velocity", "printer")?;
    config.max_accel = get_parsed(printer_section, "max_accel", "printer")?;

    // --- Process other sections ---
    let mut processed_sections = vec!["printer"];
    for (section_name, section_data) in parsed {
        if section_name.starts_with("stepper_") {
            let axis_config = build_axis_config(section_name, section_data)?;
            match section_name.replace("stepper_", "").as_str() {
                "x" => config.steppers.x = Some(axis_config),
                "y" => config.steppers.y = Some(axis_config),
                "z" => config.steppers.z = Some(axis_config),
                _ => report.warnings.push(format!(
                    "Unsupported stepper section '[{}]' was ignored.",
                    section_name
                )),
            }
            processed_sections.push(section_name);
        } else if section_name == "extruder" {
            config.extruder = Some(build_heater_config(section_name, section_data, true)?);
            processed_sections.push(section_name);
        } else if section_name == "heater_bed" {
            config.heater_bed = Some(build_heater_config(section_name, section_data, false)?);
            processed_sections.push(section_name);
        }
    }

    // --- Store unprocessed sections ---
    for (section_name, section_data) in parsed {
        if !processed_sections.contains(&section_name.as_str()) {
            config
                .other_sections
                .insert(section_name.clone(), section_data.clone());
            report.unsupported_sections.push(section_name.clone());
        }
    }

    Ok(config)
}

fn build_stepper_config(
    section_name: &str,
    data: &HashMap<String, String>,
) -> Result<StepperConfig, MigrationError> {
    Ok(StepperConfig {
        step_pin: get_required(data, "step_pin", section_name)?,
        dir_pin: get_required(data, "dir_pin", section_name)?,
        enable_pin: get_required(data, "enable_pin", section_name)?,
        microsteps: get_parsed(data, "microsteps", section_name)?,
        rotation_distance: get_parsed(data, "rotation_distance", section_name)?,
    })
}

fn build_axis_config(
    section_name: &str,
    data: &HashMap<String, String>,
) -> Result<AxisConfig, MigrationError> {
    Ok(AxisConfig {
        stepper: build_stepper_config(section_name, data)?,
        position_endstop: get_parsed(data, "position_endstop", section_name)?,
        position_max: get_parsed(data, "position_max", section_name)?,
        homing_speed: get_parsed(data, "homing_speed", section_name).unwrap_or(50.0), // Klipper defaults this
    })
}

fn build_heater_config(
    section_name: &str,
    data: &HashMap<String, String>,
    is_extruder: bool,
) -> Result<HeaterConfig, MigrationError> {
    let mut heater = HeaterConfig {
        heater_pin: get_required(data, "heater_pin", section_name)?,
        sensor_type: get_required(data, "sensor_type", section_name)?,
        sensor_pin: get_required(data, "sensor_pin", section_name)?,
        min_temp: get_parsed(data, "min_temp", section_name)?,
        max_temp: get_parsed(data, "max_temp", section_name)?,
        stepper: None,
        nozzle_diameter: None,
        filament_diameter: None,
    };

    if is_extruder {
        heater.stepper = Some(build_stepper_config(section_name, data)?);
        heater.nozzle_diameter = Some(get_parsed(data, "nozzle_diameter", section_name)?);
        heater.filament_diameter = Some(get_parsed(data, "filament_diameter", section_name)?);
    }

    Ok(heater)
}

// --- Helper functions for safe value extraction ---

fn get_required(
    section_data: &HashMap<String, String>,
    key: &str,
    section_name: &str,
) -> Result<String, MigrationError> {
    section_data
        .get(key)
        .cloned()
        .ok_or_else(|| MigrationError::MissingKey {
            section: section_name.to_string(),
            key: key.to_string(),
        })
}

fn get_parsed<T>(
    section_data: &HashMap<String, String>,
    key: &str,
    section_name: &str,
) -> Result<T, MigrationError>
where
    T: std::str::FromStr,
    T::Err: std::fmt::Display,
{
    let value_str = get_required(section_data, key, section_name)?;
    value_str.parse::<T>().map_err(|e| {
        MigrationError::InvalidValue {
            section: section_name.to_string(),
            key: key.to_string(),
            message: e.to_string(),
        }
    })
}
