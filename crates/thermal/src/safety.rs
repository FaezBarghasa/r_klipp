//! Logic for ensuring heater safety.

use num_traits::{Float, Zero};

/// The operational state of a thermal system.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum ThermalState {
    /// The system is operating normally.
    Ok,
    /// A potential issue has been detected, but it's not yet critical.
    /// This might be a slow heating or a temperature excursion.
    Warning,
    /// A critical failure has been detected, such as thermal runaway.
    /// Immediate action (e.g., hard shutdown) is required.
    Shutdown,
}

/// Defines the temperature and time limits for safety checks.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SafetyLimits<F: Float> {
    /// The maximum temperature the system should ever reach.
    pub max_temp: F,
    /// A minimum temperature gain that must be achieved within a certain time
    /// when the heater is on, to ensure it's working.
    /// (e.g., 2 degrees in 30 seconds).
    pub min_heat_gain_temp: F,
    pub min_heat_gain_time_s: F,
    /// The maximum temperature deviation from the setpoint allowed during
    /// stable operation.
    pub max_deviation: F,
}

/// Manages the safety logic for a heater.
#[derive(Debug, Clone, Copy)]
pub struct HeaterSafety<F: Float> {
    limits: SafetyLimits<F>,
    start_time: F,
    start_temp: F,
    heater_was_on: bool,
}

impl<F: Float> HeaterSafety<F> {
    /// Creates a new `HeaterSafety` monitor.
    pub fn new(limits: SafetyLimits<F>) -> Self {
        Self {
            limits,
            start_time: F::zero(),
            start_temp: F::zero(),
            heater_was_on: false,
        }
    }

    /// Updates the safety monitor with the current state.
    ///
    /// # Arguments
    /// * `current_time_s` - The current time in seconds.
    /// * `current_temp` - The current temperature.
    /// * `setpoint` - The current target temperature.
    /// * `heater_on` - Whether the heater is currently powered.
    ///
    /// # Returns
    /// The `ThermalState` indicating the system's safety status.
    pub fn update(&mut self, current_time_s: F, current_temp: F, setpoint: F, heater_on: bool) -> ThermalState {
        // Absolute max temperature check
        if current_temp > self.limits.max_temp {
            return ThermalState::Shutdown;
        }

        // Check if the heater just turned on
        if heater_on && !self.heater_was_on {
            self.start_time = current_time_s;
            self.start_temp = current_temp;
        }
        self.heater_was_on = heater_on;

        if heater_on {
            // Thermal runaway check: Is it heating up as expected?
            let elapsed_time = current_time_s - self.start_time;
            if elapsed_time > self.limits.min_heat_gain_time_s {
                let temp_gain = current_temp - self.start_temp;
                if temp_gain < self.limits.min_heat_gain_temp {
                    return ThermalState::Shutdown; // Not heating up, possible thermistor failure or broken heater
                }
            }
        } else {
            // Reset the heating check timer when the heater is off
            self.start_time = current_time_s;
            self.start_temp = current_temp;
        }

        // Check for temperature deviation from setpoint if we are close to it
        if !setpoint.is_zero() && (current_temp - setpoint).abs() > self.limits.max_deviation {
            // Only trigger if we should be at temperature
            if (self.start_temp - setpoint).abs() < self.limits.max_deviation {
                return ThermalState::Warning;
            }
        }

        ThermalState::Ok
    }
}
