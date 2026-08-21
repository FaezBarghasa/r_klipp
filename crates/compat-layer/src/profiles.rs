//! Multi-Target Profile Schema Validator for 3D Printers, PnP/PIP, and CNC Machines.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MachineType {
    #[serde(rename = "3d_printer")]
    ThreeDPrinter,
    #[serde(rename = "pnp")]
    PnpPip,
    #[serde(rename = "cnc")]
    Cnc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UniversalMachineConfig {
    pub machine_type: MachineType,
    pub name: String,
    pub max_velocity: f64,
    pub max_accel: f64,
    pub max_jerk: Option<f64>,

    // 3D Printer Specific
    pub extruder: Option<ExtruderConfig>,
    pub heater_bed: Option<HeaterBedConfig>,

    // PnP / PIP Specific
    pub pnp: Option<PnpConfig>,

    // CNC Specific
    pub cnc: Option<CncConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtruderConfig {
    pub step_pin: String,
    pub dir_pin: String,
    pub enable_pin: String,
    pub heater_pin: String,
    pub sensor_pin: String,
    pub max_temp: f64,
    pub pressure_advance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaterBedConfig {
    pub heater_pin: String,
    pub sensor_pin: String,
    pub max_temp: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnpConfig {
    pub head_1_z_pin: String,
    pub head_2_z_pin: String,
    pub vacuum_valve_pin: String,
    pub camera_trigger_pin: String,
    pub can_feeder_bus: String,
    pub stall_current_threshold_mA: u16,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CncConfig {
    pub spindle_pwm_pin: Option<String>,
    pub spindle_modbus_port: Option<String>,
    pub max_spindle_rpm: u32,
    pub coolant_flood_pin: Option<String>,
    pub touch_probe_pin: String,
    pub enclosure_door_interlock_pin: Option<String>,
    pub has_5axis_rtcp: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProfileValidationError {
    MissingExtruderSection,
    MissingPnpSection,
    MissingCncSection,
    InvalidSpeedLimits,
    MissingTouchProbe,
}

impl UniversalMachineConfig {
    pub fn validate_profile(&self) -> Result<(), ProfileValidationError> {
        if self.max_velocity <= 0.0 || self.max_accel <= 0.0 {
            return Err(ProfileValidationError::InvalidSpeedLimits);
        }

        match self.machine_type {
            MachineType::ThreeDPrinter => {
                if self.extruder.is_none() {
                    return Err(ProfileValidationError::MissingExtruderSection);
                }
            }
            MachineType::PnpPip => {
                if self.pnp.is_none() {
                    return Err(ProfileValidationError::MissingPnpSection);
                }
            }
            MachineType::Cnc => {
                if let Some(ref cnc) = self.cnc {
                    if cnc.touch_probe_pin.is_empty() {
                        return Err(ProfileValidationError::MissingTouchProbe);
                    }
                } else {
                    return Err(ProfileValidationError::MissingCncSection);
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_3d_printer_profile_validation() {
        let cfg = UniversalMachineConfig {
            machine_type: MachineType::ThreeDPrinter,
            name: "Voron2.4".to_string(),
            max_velocity: 300.0,
            max_accel: 5000.0,
            max_jerk: Some(50000.0),
            extruder: Some(ExtruderConfig {
                step_pin: "PE2".to_string(),
                dir_pin: "PE3".to_string(),
                enable_pin: "PD4".to_string(),
                heater_pin: "PA2".to_string(),
                sensor_pin: "PA0".to_string(),
                max_temp: 300.0,
                pressure_advance: Some(0.04),
            }),
            heater_bed: None,
            pnp: None,
            cnc: None,
        };

        assert!(cfg.validate_profile().is_ok());
    }

    #[test]
    fn test_pnp_profile_validation() {
        let cfg = UniversalMachineConfig {
            machine_type: MachineType::PnpPip,
            name: "LumenPnP".to_string(),
            max_velocity: 800.0,
            max_accel: 15000.0,
            max_jerk: None,
            extruder: None,
            heater_bed: None,
            pnp: Some(PnpConfig {
                head_1_z_pin: "PE4".to_string(),
                head_2_z_pin: "PE5".to_string(),
                vacuum_valve_pin: "PA1".to_string(),
                camera_trigger_pin: "PB0".to_string(),
                can_feeder_bus: "can0".to_string(),
                stall_current_threshold_mA: 1200,
            }),
            cnc: None,
        };

        assert!(cfg.validate_profile().is_ok());
    }

    #[test]
    fn test_cnc_profile_validation() {
        let cfg = UniversalMachineConfig {
            machine_type: MachineType::Cnc,
            name: "PocketNC_5Axis".to_string(),
            max_velocity: 150.0,
            max_accel: 2000.0,
            max_jerk: None,
            extruder: None,
            heater_bed: None,
            pnp: None,
            cnc: Some(CncConfig {
                spindle_pwm_pin: Some("PA8".to_string()),
                spindle_modbus_port: Some("/dev/ttyUSB0".to_string()),
                max_spindle_rpm: 24000,
                coolant_flood_pin: Some("PC9".to_string()),
                touch_probe_pin: "PC13".to_string(),
                enclosure_door_interlock_pin: Some("PD2".to_string()),
                has_5axis_rtcp: true,
            }),
        };

        assert!(cfg.validate_profile().is_ok());
    }
}
