#![no_std]

use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

pub const ABSOLUTE_MAX_TEMPERATURE: f32 = 320.0;

/// Reasons for an emergency stop.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum EStopReason {
    None = 0,
    ThermalRunaway = 1,
    TemperatureOutOfBounds = 2,
    MotorFault = 3,
    HostCommand = 4,
    Other = 255,
}

impl From<u8> for EStopReason {
    fn from(value: u8) -> Self {
        match value {
            1 => EStopReason::ThermalRunaway,
            2 => EStopReason::TemperatureOutOfBounds,
            3 => EStopReason::MotorFault,
            4 => EStopReason::HostCommand,
            _ => EStopReason::Other,
        }
    }
}

/// Global E-Stop state accessible across the MCU.
pub struct EStopState {
    /// True if an E-Stop is currently active.
    is_active: AtomicBool,
    /// The reason for the E-Stop.
    reason: AtomicU8,
}

impl EStopState {
    /// Creates a new EStopState, initially inactive.
    pub const fn new() -> Self {
        Self {
            is_active: AtomicBool::new(false),
            reason: AtomicU8::new(EStopReason::None as u8),
        }
    }

    /// Triggers an E-Stop with the given reason.
    pub fn trigger(&self, reason: EStopReason) {
        self.reason.store(reason as u8, Ordering::Release);
        self.is_active.store(true, Ordering::Release);
        // TODO: Potentially add a mechanism to signal other modules immediately
        // e.g., via a Waker or a global event queue.
    }

    /// Clears the E-Stop state.
    pub fn clear(&self) {
        self.is_active.store(false, Ordering::Release);
        self.reason.store(EStopReason::None as u8, Ordering::Release);
    }

    /// Checks if an E-Stop is currently active.
    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Acquire)
    }

    /// Gets the current E-Stop reason.
    pub fn get_reason(&self) -> EStopReason {
        self.reason.load(Ordering::Acquire).into()
    }
}

/// Global static instance of the EStopState.
/// This allows any part of the firmware to trigger or check the E-Stop status.
static GLOBAL_ESTOP_STATE: EStopState = EStopState::new();

/// Public function to trigger an E-Stop.
pub fn trigger_e_stop(reason: EStopReason) {
    GLOBAL_ESTOP_STATE.trigger(reason);
}

/// Public function to clear an E-Stop.
pub fn clear_e_stop() {
    GLOBAL_ESTOP_STATE.clear();
}

/// Public function to check if an E-Stop is active.
pub fn is_e_stop_active() -> bool {
    GLOBAL_ESTOP_STATE.is_active()
}

/// Public function to get the current E-Stop reason.
pub fn get_e_stop_reason() -> EStopReason {
    GLOBAL_ESTOP_STATE.get_reason()
}


/// Configuration for the RunawayWatchdog.
pub struct RunawayWatchdogConfig {
    /// Minimum temperature threshold (e.g., -200.0 C) to detect open circuit.
    pub min_temp_threshold: f32,
    /// Maximum temperature threshold (e.g., 500.0 C) to detect short circuit.
    pub max_temp_threshold: f32,
    /// Maximum allowed temperature change rate in C/second when heater is active.
    pub max_temp_change_rate: f32,
    /// Minimum time in milliseconds for temperature change rate calculation.
    pub min_time_for_rate_ms: u32,
}

/// Monitors temperature for runaway conditions and triggers an E-Stop if detected.
pub struct RunawayWatchdog {
    config: RunawayWatchdogConfig,
    last_temp: f32,
    last_timestamp_ms: u32,
    e_stop_triggered: bool, // This local flag is now redundant if using GLOBAL_ESTOP_STATE
}

impl RunawayWatchdog {
    pub fn new(config: RunawayWatchdogConfig, initial_temp: f32, initial_timestamp_ms: u32) -> Self {
        Self {
            config,
            last_temp: initial_temp,
            last_timestamp_ms: initial_timestamp_ms,
            e_stop_triggered: false, // Will rely on GLOBAL_ESTOP_STATE
        }
    }

    /// Checks the current temperature for runaway conditions.
    /// Returns `true` if an E-Stop should be triggered.
    pub fn check_temperature(&mut self, current_temp: f32, current_timestamp_ms: u32, heater_active: bool) -> bool {
        if is_e_stop_active() {
            return true; // Global E-Stop already active
        }

        // 1. Open/Short Circuit Detection (ADC rail hits)
        if current_temp < self.config.min_temp_threshold {
            trigger_e_stop(EStopReason::TemperatureOutOfBounds);
            return true;
        }
        if current_temp > self.config.max_temp_threshold {
            trigger_e_stop(EStopReason::TemperatureOutOfBounds);
            return true;
        }

        // 2. Slope-based Thermal Runaway Detection
        let dt_ms = current_timestamp_ms.saturating_sub(self.last_timestamp_ms);

        if heater_active && dt_ms >= self.config.min_time_for_rate_ms {
            let temp_change = current_temp - self.last_temp;
            let time_change_s = dt_ms as f32 / 1000.0;

            if time_change_s > 0.0 {
                let temp_change_rate = temp_change / time_change_s;

                // If temperature is rising too fast (unlikely for runaway, but good to catch)
                if temp_change_rate > self.config.max_temp_change_rate {
                     trigger_e_stop(EStopReason::ThermalRunaway); // Or a more specific "TempRateTooHigh"
                     return true;
                }

                // If heater is active but temperature is not rising (or dropping)
                // This is the core of slope-based runaway detection.
                // This needs refinement with a proper minimum expected rise.
                if heater_active && temp_change_rate <= 0.0 { // Placeholder: needs a proper minimum rise threshold
                    trigger_e_stop(EStopReason::ThermalRunaway);
                    return true;
                }
            }
        }

        // Update for next iteration
        self.last_temp = current_temp;
        self.last_timestamp_ms = current_timestamp_ms;

        false // No E-Stop triggered by this watchdog instance
    }

    /// Returns true if an E-Stop has been triggered.
    pub fn is_e_stop_triggered(&self) -> bool {
        is_e_stop_active()
    }

    /// Resets the E-Stop state.
    pub fn reset_e_stop(&mut self) {
        clear_e_stop();
    }
}
