// AI-generated comment:
// This file was modified by an AI assistant to review and document critical section optimization.
// Source files for context: crates/klipper-mcu-firmware/src/heater.rs

//! # PID Heater Control
//!
//! This module implements a PID control loop for the 3D printer's heaters,
//! using fixed-point arithmetic for real-time performance. It includes safety
//! interlocks to prevent overheating and to shut down the heater in case of a
//! sensor failure.
//!
//! ## Fixed-Point PID
//!
//! The PID controller continuously adjusts the heater's power output (via PWM) to
//! maintain the target temperature. The controller's parameters (P, I, and D) and
//! all internal calculations use `Fixed16_16` numbers to avoid floating-point
//! math, ensuring deterministic execution times.
//!
//! ## Safety
//!
//! This task integrates with the global `SafetyMonitor`. It checks the emergency
//! stop flag on every iteration and will immediately disable the heater if a
//! global shutdown is triggered. It is also responsible for feeding temperature
//! data to the thermal runaway protection system.

use crate::fixed_point::Fixed16_16;
use crate::safety::SafetyMonitor;
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::simple_pwm::{PwmPin, SimplePwm};
use embassy_stm32::timer::Channel;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::mutex::Mutex;
use embassy_time::{Duration, Ticker};

/// Represents the shared state for a single heater.
pub struct HeaterSharedState {
    pub target_temp: Mutex<CriticalSectionRawMutex, Fixed16_16>,
    pub current_temp: Mutex<CriticalSectionRawMutex, Fixed16_16>,
}

impl HeaterSharedState {
    pub const fn new() -> Self {
        Self {
            target_temp: Mutex::new(Fixed16_16::ZERO),
            current_temp: Mutex::new(Fixed16_16::ZERO),
        }
    }
}

/// A PID controller using 16.16 fixed-point arithmetic.
pub struct PidController {
    kp: Fixed16_16,
    ki: Fixed16_16,
    kd: Fixed16_16,
    integral: Fixed16_16,
    prev_error: Fixed16_16,
    // Output is clamped to 0..=max_duty (e.g., 0..=65535 for a 16-bit PWM)
    output_max: Fixed16_16,
    integral_max: Fixed16_16,
}

impl PidController {
    /// Creates a new `PidController`.
    ///
    /// # Arguments
    /// * `kp`, `ki`, `kd` - PID gains.
    /// * `output_max` - The maximum value of the controller output (e.g., PWM max duty).
    pub fn new(kp: f32, ki: f32, kd: f32, output_max: u16) -> Self {
        let ki_fixed = Fixed16_16::from_float(ki);
        let output_max_fixed = Fixed16_16::from_float(output_max as f32);

        // Pre-calculate integral limit for anti-windup.
        // Avoid division by zero if Ki is zero.
        let integral_max = if ki_fixed > Fixed16_16::ZERO {
            output_max_fixed / ki_fixed
        } else {
            Fixed16_16::ZERO
        };

        Self {
            kp: Fixed16_16::from_float(kp),
            ki: ki_fixed,
            kd: Fixed16_16::from_float(kd),
            integral: Fixed16_16::ZERO,
            prev_error: Fixed16_16::ZERO,
            output_max: output_max_fixed,
            integral_max,
        }
    }

    /// Resets the controller's internal state.
    pub fn reset(&mut self) {
        self.integral = Fixed16_16::ZERO;
        self.prev_error = Fixed16_16::ZERO;
    }

    /// Calculates the next control output.
    ///
    /// # Arguments
    /// * `error` - The difference between the target and current temperature (`target - current`).
    /// * `dt` - The time delta since the last update in seconds.
    pub fn update(&mut self, error: Fixed16_16, dt: Fixed16_16) -> u16 {
        // Proportional term
        let p_term = self.kp * error;

        // Integral term with anti-windup
        self.integral = self.integral + (error * dt);
        if self.integral > self.integral_max {
            self.integral = self.integral_max;
        } else if self.integral < -self.integral_max {
            self.integral = -self.integral_max;
        }
        let i_term = self.ki * self.integral;

        // Derivative term
        let derivative = (error - self.prev_error) / dt;
        self.prev_error = error;
        let d_term = self.kd * derivative;

        // Combine terms and clamp output
        let mut output = p_term + i_term + d_term;
        if output < Fixed16_16::ZERO {
            output = Fixed16_16::ZERO;
        } else if output > self.output_max {
            output = self.output_max;
        }

        output.0 as u16
    }
}

/// The main heater control task.
///
/// This task runs a PID control loop to maintain the target temperature for a heater.
///
/// # Arguments
/// * `heater_id` - A unique identifier for this heater (e.g., 0 for extruder).
/// * `pwm` - An initialized `SimplePwm` instance for the heater's PWM pin.
/// * `channel` - The PWM channel to use.
/// * `state` - The shared state for this heater.
/// * `safety` - A reference to the global `SafetyMonitor`.
/// * `update_freq_hz` - How often the PID loop should run.
#[embassy_executor::task]
pub async fn heater_task<'a, T: embassy_stm32::timer::Instance>(
    heater_id: usize,
    mut pwm: SimplePwm<'a, T>,
    channel: Channel,
    state: &'static HeaterSharedState,
    safety: &'static Mutex<CriticalSectionRawMutex, SafetyMonitor<'a, 4, 4>>, // Assuming max 4 heaters/tasks
    update_freq_hz: u32,
) {
    defmt::info!("Heater task {} started. Update frequency: {} Hz", heater_id, update_freq_hz);

    let max_duty = pwm.get_max_duty();
    pwm.enable(channel);

    // TODO: Load these from config
    let mut pid = PidController::new(10.0, 0.1, 8.0, max_duty);

    let interval = Duration::from_hz(update_freq_hz as u64);
    let dt = Fixed16_16::from_float(1.0 / update_freq_hz as f32);
    let mut ticker = Ticker::every(interval);

    loop {
        // AI-generated note: The following code already adheres to the principle of minimizing
        // critical section duration. The `.lock().await` calls on Embassy's async mutexes
        // acquire the lock, perform a quick operation (dereference/copy or a fast function call),
        // and then immediately release the lock. The long-running PID calculation (`pid.update`)
        // is performed on local variables (`target`, `current`) outside of any critical section.
        // This structure is efficient and safe for an async context. No refactoring is needed.

        // Check for emergency stop first
        if safety.lock().await.is_emergency_stop_active() {
            pwm.set_duty(channel, 0);
            defmt::warn!("Heater {}: E-STOP active, shutting down.", heater_id);
            // Task continues to run but heater remains off.
            // A more robust implementation might suspend the task.
            ticker.next().await;
            continue;
        }

        let target = *state.target_temp.lock().await;
        let current = *state.current_temp.lock().await;

        // Feed the safety monitor with the latest temperature
        safety.lock().await.check_thermal_state(heater_id, current.to_float());

        let duty = if target.0 <= 0 {
            pid.reset();
            0
        } else {
            let error = target - current;
            pid.update(error, dt)
        };

        pwm.set_duty(channel, duty);

        ticker.next().await;
    }
}
