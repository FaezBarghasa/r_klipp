// File path: crates/klipper-mcu-firmware/src/safety.rs
#![deny(clippy::all)]
#![deny(warnings)]

//! # Safety Monitoring System
//!
//! This module provides the core safety mechanisms for the firmware, including
//! thermal protection, an independent watchdog, and an emergency stop facility.
//! Its primary goal is to prevent hardware damage and ensure operator safety
//! in the event of a fault.
//!
//! ## Safety
//!
//! This is a life-safety critical module.
//!
//! - **Watchdog**: Utilizes the hardware's Independent Watchdog (IWDG) to reset
//!   the MCU if the main firmware loop hangs. The watchdog must be "fed"
//!   periodically to prevent a reset.
//! - **Thermal Runaway**: Monitors the rate of temperature change for each heater.
//!   If a heater's temperature rises faster than a configured limit (e.g., 5°C/sec),
//!   it's considered a dangerous thermal runaway, and an emergency stop is triggered.
//! - **Sensor Failure**: Checks for temperatures outside a plausible range (e.g.,
//!   <-50°C or >300°C), which typically indicates a shorted or disconnected thermistor.
//!   This also triggers an emergency stop.
//! - **Emergency Stop**: A globally accessible, atomic flag (`emergency_stop_active`)
//!   provides a fast, interrupt-safe way to signal a shutdown condition. Higher-level
//!   application code is responsible for polling this flag and acting on it immediately
//!   by disabling all heaters, motors, and other outputs.

use core::sync::atomic::{AtomicBool, Ordering};
use embassy_stm32::wdg::IndependentWatchdog;
use embassy_time::{Duration, Instant};

/// Represents a specific safety-related fault.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum SafetyError {
    /// Temperature rose faster than the configured maximum rate.
    ThermalRunaway { heater_id: usize, rate_of_change: f32 },
    /// Temperature is below the configured minimum limit, suggesting a sensor short.
    TempTooLow { heater_id: usize, temp: f32 },
    /// Temperature is above the configured maximum limit, suggesting a sensor disconnect.
    TempTooHigh { heater_id: usize, temp: f32 },
    /// A stepper driver reported a fault condition (e.g., overtemperature).
    StepperDriverFault { driver_mask: u8 },
    /// A critical task failed to check in within its deadline.
    TaskStalled { task_id: usize },
}

/// Configuration and state for monitoring a single thermal zone.
#[derive(Clone, Copy)]
pub struct ThermalMonitor {
    /// The maximum safe rate of temperature increase in degrees Celsius per second.
    max_rate_celsius_per_sec: f32,
    /// The minimum plausible temperature reading in degrees Celsius.
    min_temp_limit: f32,
    /// The maximum plausible temperature reading in degrees Celsius.
    max_temp_limit: f32,
    /// The temperature recorded at the last check.
    last_temp: f32,
    /// The timestamp of the last check.
    last_check_time: Instant,
}

impl ThermalMonitor {
    /// Creates a new thermal monitor with the specified limits.
    pub const fn new(
        max_rate_celsius_per_sec: f32,
        min_temp_limit: f32,
        max_temp_limit: f32,
        initial_temp: f32,
    ) -> Self {
        Self {
            max_rate_celsius_per_sec,
            min_temp_limit,
            max_temp_limit,
            last_temp: initial_temp,
            // Will be updated on the first check
            last_check_time: Instant::from_ticks(0),
        }
    }

    /// Checks the current temperature against the safety limits.
    ///
    /// # Returns
    /// `Err(SafetyError)` if a safety limit is violated.
    pub fn check(&mut self, heater_id: usize, current_temp: f32) -> Result<(), SafetyError> {
        let now = Instant::now();

        // 1. Check for min/max temperature sensor failure.
        if current_temp < self.min_temp_limit {
            return Err(SafetyError::TempTooLow { heater_id, temp: current_temp });
        }
        if current_temp > self.max_temp_limit {
            return Err(SafetyError::TempTooHigh { heater_id, temp: current_temp });
        }

        // 2. Check for thermal runaway (rate of change).
        // Skip the first check as we need a time delta.
        if self.last_check_time.as_ticks() > 0 {
            let delta_time_s = now.duration_since(self.last_check_time).as_micros() as f32 / 1_000_000.0;
            // Avoid division by zero and nonsensical checks on very short intervals.
            if delta_time_s > 0.1 {
                let delta_temp = current_temp - self.last_temp;
                let rate_of_change = delta_temp / delta_time_s;

                if rate_of_change > self.max_rate_celsius_per_sec {
                    return Err(SafetyError::ThermalRunaway { heater_id, rate_of_change });
                }
            }
        }

        // Update state for the next check.
        self.last_temp = current_temp;
        self.last_check_time = now;

        Ok(())
    }
}

/// The main safety supervisor for the entire MCU.
/// It aggregates all safety-critical components.
pub struct SafetyMonitor<'a, const NUM_HEATERS: usize, const NUM_TASKS: usize> {
    thermal_monitors: [ThermalMonitor; NUM_HEATERS],
    watchdog: IndependentWatchdog<'a>,
    /// Global flag indicating an emergency stop has been triggered.
    /// This MUST be polled by high-level tasks to shut down hardware.
    emergency_stop_active: AtomicBool,
    /// Timestamps of the last check-in for each monitored task.
    last_check_in: [Instant; NUM_TASKS],
    /// The deadline for each task to check in.
    task_deadlines: [Duration; NUM_TASKS],
}

impl<'a, const NUM_HEATERS: usize, const NUM_TASKS: usize> SafetyMonitor<'a, NUM_HEATERS, NUM_TASKS> {
    /// Creates a new `SafetyMonitor`.
    pub fn new(
        thermal_monitors: [ThermalMonitor; NUM_HEATERS],
        mut watchdog: IndependentWatchdog<'a>,
        task_deadlines: [Duration; NUM_TASKS],
    ) -> Self {
        watchdog.unleash();
        Self {
            thermal_monitors,
            watchdog,
            emergency_stop_active: AtomicBool::new(false),
            last_check_in: [Instant::now(); NUM_TASKS],
            task_deadlines,
        }
    }

    /// Checks the thermal state of a specific heater.
    /// If a fault is detected, it automatically triggers a global emergency stop.
    pub fn check_thermal_state(&mut self, heater_id: usize, temp: f32) {
        if let Some(monitor) = self.thermal_monitors.get_mut(heater_id) {
            if let Err(e) = monitor.check(heater_id, temp) {
                self.trigger_emergency_stop(e);
            }
        }
    }

    /// Checks the fault status of stepper drivers.
    /// `fault_pin_states` is a bitmask where each bit corresponds to a driver's fault pin.
    /// A high bit indicates a fault.
    pub fn check_stepper_faults(&mut self, fault_pin_states: u8) {
        if fault_pin_states != 0 {
            self.trigger_emergency_stop(SafetyError::StepperDriverFault { driver_mask: fault_pin_states });
        }
    }

    /// Allows a task to "check-in", resetting its software watchdog timer.
    pub fn task_check_in(&mut self, task_id: usize) {
        if let Some(check_in) = self.last_check_in.get_mut(task_id) {
            *check_in = Instant::now();
        }
    }

    /// Iterates through all monitored tasks and checks if any have missed their deadline.
    pub fn check_task_stalls(&mut self) {
        let now = Instant::now();
        for i in 0..NUM_TASKS {
            if now.duration_since(self.last_check_in[i]) > self.task_deadlines[i] {
                self.trigger_emergency_stop(SafetyError::TaskStalled { task_id: i });
            }
        }
    }

    /// Triggers a global emergency stop.
    pub fn trigger_emergency_stop(&mut self, reason: SafetyError) {
        if self.emergency_stop_active.swap(true, Ordering::SeqCst) == false {
            defmt::error!("EMERGENCY STOP TRIGGERED: {:?}", reason);
        }
    }

    /// Returns `true` if an emergency stop is currently active.
    #[inline]
    pub fn is_emergency_stop_active(&self) -> bool {
        self.emergency_stop_active.load(Ordering::SeqCst)
    }

    /// "Feeds" the independent watchdog.
    #[inline]
    pub fn feed_watchdog(&mut self) {
        self.watchdog.feed();
    }
}

