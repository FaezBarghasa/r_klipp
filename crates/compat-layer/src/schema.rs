//! Strict Schema Validation for Printer Configuration.

use crate::models::{PrinterConfig, AxisConfig, HeaterConfig};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationError {
    InvalidKinematics(String),
    NonPositiveMaxVelocity(u32),
    NonPositiveMaxAccel(u32),
    MissingAxis(String),
    InvalidAxisConfig { axis: String, reason: String },
    InvalidHeaterConfig { heater: String, reason: String },
    EmptyPinAssignment { section: String, pin_name: String },
}

pub fn validate_printer_config(config: &PrinterConfig) -> Result<(), Vec<ValidationError>> {
    let mut errors = Vec::new();

    // 1. Kinematics validation
    let kin = config.kinematics.to_lowercase();
    if kin != "cartesian" && kin != "corexy" && kin != "delta" {
        errors.push(ValidationError::InvalidKinematics(config.kinematics.clone()));
    }

    // 2. Velocity & Acceleration
    if config.max_velocity <= 0.0 {
        errors.push(ValidationError::NonPositiveMaxVelocity(config.max_velocity as u32));
    }
    if config.max_accel <= 0.0 {
        errors.push(ValidationError::NonPositiveMaxAccel(config.max_accel as u32));
    }

    // 3. Axes validation
    validate_axis("X", &config.steppers.x, &mut errors);
    validate_axis("Y", &config.steppers.y, &mut errors);
    validate_axis("Z", &config.steppers.z, &mut errors);

    // 4. Heater validation
    if let Some(ref ext) = config.extruder {
        validate_heater("extruder", ext, &mut errors);
    }
    if let Some(ref bed) = config.heater_bed {
        validate_heater("heater_bed", bed, &mut errors);
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

fn validate_axis(name: &str, axis_opt: &Option<AxisConfig>, errors: &mut Vec<ValidationError>) {
    match axis_opt {
        None => errors.push(ValidationError::MissingAxis(name.to_string())),
        Some(axis) => {
            if axis.position_max <= axis.position_endstop {
                errors.push(ValidationError::InvalidAxisConfig {
                    axis: name.to_string(),
                    reason: "position_max must be greater than position_endstop".to_string(),
                });
            }
            if axis.homing_speed <= 0.0 {
                errors.push(ValidationError::InvalidAxisConfig {
                    axis: name.to_string(),
                    reason: "homing_speed must be positive".to_string(),
                });
            }
            if axis.stepper.microsteps == 0 {
                errors.push(ValidationError::InvalidAxisConfig {
                    axis: name.to_string(),
                    reason: "microsteps must be >= 1".to_string(),
                });
            }
            if axis.stepper.rotation_distance <= 0.0 {
                errors.push(ValidationError::InvalidAxisConfig {
                    axis: name.to_string(),
                    reason: "rotation_distance must be positive".to_string(),
                });
            }
            if axis.stepper.step_pin.trim().is_empty() {
                errors.push(ValidationError::EmptyPinAssignment {
                    section: format!("stepper_{}", name.to_lowercase()),
                    pin_name: "step_pin".to_string(),
                });
            }
            if axis.stepper.dir_pin.trim().is_empty() {
                errors.push(ValidationError::EmptyPinAssignment {
                    section: format!("stepper_{}", name.to_lowercase()),
                    pin_name: "dir_pin".to_string(),
                });
            }
        }
    }
}

fn validate_heater(name: &str, heater: &HeaterConfig, errors: &mut Vec<ValidationError>) {
    if heater.min_temp >= heater.max_temp {
        errors.push(ValidationError::InvalidHeaterConfig {
            heater: name.to_string(),
            reason: format!("min_temp ({}) must be strictly less than max_temp ({})", heater.min_temp, heater.max_temp),
        });
    }
    if heater.heater_pin.trim().is_empty() {
        errors.push(ValidationError::EmptyPinAssignment {
            section: name.to_string(),
            pin_name: "heater_pin".to_string(),
        });
    }
    if heater.sensor_pin.trim().is_empty() {
        errors.push(ValidationError::EmptyPinAssignment {
            section: name.to_string(),
            pin_name: "sensor_pin".to_string(),
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{AxisConfig, StepperConfig, StepperAxes};

    #[test]
    fn test_valid_config() {
        let config = PrinterConfig {
            kinematics: "cartesian".to_string(),
            max_velocity: 300.0,
            max_accel: 3000.0,
            steppers: StepperAxes {
                x: Some(AxisConfig {
                    stepper: StepperConfig {
                        step_pin: "PE2".to_string(),
                        dir_pin: "PB8".to_string(),
                        enable_pin: "PB9".to_string(),
                        microsteps: 16,
                        rotation_distance: 40.0,
                    },
                    position_endstop: 0.0,
                    position_max: 235.0,
                    homing_speed: 50.0,
                }),
                y: Some(AxisConfig {
                    stepper: StepperConfig {
                        step_pin: "PB4".to_string(),
                        dir_pin: "PB3".to_string(),
                        enable_pin: "PD2".to_string(),
                        microsteps: 16,
                        rotation_distance: 40.0,
                    },
                    position_endstop: 0.0,
                    position_max: 235.0,
                    homing_speed: 50.0,
                }),
                z: Some(AxisConfig {
                    stepper: StepperConfig {
                        step_pin: "PD7".to_string(),
                        dir_pin: "PD6".to_string(),
                        enable_pin: "PD5".to_string(),
                        microsteps: 16,
                        rotation_distance: 8.0,
                    },
                    position_endstop: 0.0,
                    position_max: 250.0,
                    homing_speed: 10.0,
                }),
            },
            extruder: Some(HeaterConfig {
                heater_pin: "PA0".to_string(),
                sensor_type: "EPCOS 100K".to_string(),
                sensor_pin: "PC0".to_string(),
                min_temp: 0.0,
                max_temp: 275.0,
                stepper: None,
                nozzle_diameter: Some(0.4),
                filament_diameter: Some(1.75),
            }),
            heater_bed: None,
            other_sections: Default::default(),
        };

        assert!(validate_printer_config(&config).is_ok());
    }
}
