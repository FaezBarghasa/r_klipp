use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum MachineProfile {
    FdmPrinter,
    CncRouter,
    FiveAxisCnc,
    WireEdm,
    PickAndPlace,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct McuSelection {
    pub name: String,
    pub target_triple: String,
    pub memory_x_script: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum AxisType {
    Linear,
    Rotary,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub enum DriverType {
    TMC2209,
    CiA402,
    FOC,
    StepDir,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct AxisConfig {
    pub axis_type: AxisType,
    pub steps_per_mm: f32,
    pub max_velocity: f32,
    pub max_acceleration: f32,
    pub driver_type: DriverType,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone, Default)]
pub struct PinMapping {
    pub pins: HashMap<String, String>,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct RklippConfig {
    pub machine_profile: MachineProfile,
    pub mcu_selection: McuSelection,
    pub axes: Vec<AxisConfig>,
    pub pin_mapping: PinMapping,
}

impl RklippConfig {
    pub fn validate(&self) -> Result<(), Vec<String>> {
        let mut errors = Vec::new();

        // Validate axis count for the selected machine profile
        let expected_axes = match self.machine_profile {
            MachineProfile::FdmPrinter => 3..=5, // X, Y, Z, E0, E1
            MachineProfile::CncRouter => 3..=4,
            MachineProfile::FiveAxisCnc => 5..=6,
            MachineProfile::WireEdm => 4..=5,
            MachineProfile::PickAndPlace => 4..=6,
        };
        if !expected_axes.contains(&self.axes.len()) {
            errors.push(format!(
                "Invalid number of axes for {:?}. Expected {:?}, found {}.",
                self.machine_profile, expected_axes, self.axes.len()
            ));
        }

        // Validate pin mapping for conflicts
        let mut used_pins = HashSet::new();
        for (function, pin) in &self.pin_mapping.pins {
            if !used_pins.insert(pin.clone()) {
                errors.push(format!("Pin conflict: Pin '{}' is used for more than one function (e.g., '{}').", pin, function));
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }
}

impl Default for RklippConfig {
    fn default() -> Self {
        Self {
            machine_profile: MachineProfile::FdmPrinter,
            mcu_selection: McuSelection {
                name: "Stm32f407ve".to_string(),
                target_triple: "thumbv7em-none-eabihf".to_string(),
                memory_x_script: "memory.x".to_string(),
            },
            axes: vec![
                AxisConfig {
                    axis_type: AxisType::Linear,
                    steps_per_mm: 80.0,
                    max_velocity: 500.0,
                    max_acceleration: 3000.0,
                    driver_type: DriverType::TMC2209,
                }; 3
            ],
            pin_mapping: PinMapping::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pin_conflict_validation() {
        let mut config = RklippConfig::default();
        config.pin_mapping.pins.insert("X_STEP_PIN".to_string(), "PA5".to_string());
        config.pin_mapping.pins.insert("Y_STEP_PIN".to_string(), "PA5".to_string());
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Pin conflict: Pin 'PA5'"));
    }

    #[test]
    fn test_axis_count_validation() {
        let mut config = RklippConfig::default();
        config.machine_profile = MachineProfile::CncRouter;
        config.axes = vec![AxisConfig {
            axis_type: AxisType::Linear,
            steps_per_mm: 80.0,
            max_velocity: 500.0,
            max_acceleration: 3000.0,
            driver_type: DriverType::StepDir,
        }; 2]; // Too few axes for a CNC router
        let result = config.validate();
        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Invalid number of axes"));
    }

    #[test]
    fn test_valid_config() {
        let mut config = RklippConfig::default();
        config.machine_profile = MachineProfile::CncRouter;
        config.axes = vec![AxisConfig {
            axis_type: AxisType::Linear,
            steps_per_mm: 80.0,
            max_velocity: 500.0,
            max_acceleration: 3000.0,
            driver_type: DriverType::StepDir,
        }; 3];
        config.pin_mapping.pins.insert("X_STEP_PIN".to_string(), "PA5".to_string());
        config.pin_mapping.pins.insert("Y_STEP_PIN".to_string(), "PA6".to_string());
        let result = config.validate();
        assert!(result.is_ok());
    }
}
