#![no_std]
use libm;
use crate::tuner_trait::{AutoTuner, TunerState};

/// A motion auto-tuner based on Step-Response Identification.
///
/// Unlike thermal systems, motion systems cannot use ON/OFF relays.
/// This tuner injects a velocity/acceleration step, measures the
/// system's rise time and steady-state error to estimate equivalent
/// inertia and friction, then calculates optimal PI/PID gains using
/// the Symmetrical Optimum (SO) method.
pub struct MotionStepTuner {
    // Configuration
    max_excitation: f32, // Max current/acceleration to apply during step
    target_step: f32,    // The size of the step to inject

    // State Machine
    state: TunerState,
    phase: MotionPhase,

    // Data Collection
    start_time: f32,
    initial_measurement: f32,
    peak_overshoot: f32,
    rise_time_90: f32,

    // Results
    kp: f32,
    ki: f32,
    kd: f32,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum MotionPhase {
    Idle,
    InjectingStep,
    Analyzing,
}

impl MotionStepTuner {
    pub fn new(max_excitation: f32, target_step: f32) -> Self {
        Self {
            max_excitation,
            target_step,
            state: TunerState::Idle,
            phase: MotionPhase::Idle,
            start_time: 0.0,
            initial_measurement: 0.0,
            peak_overshoot: 0.0,
            rise_time_90: 0.0,
            kp: 0.0,
            ki: 0.0,
            kd: 0.0,
        }
    }
}

impl AutoTuner for MotionStepTuner {
    fn update(&mut self, setpoint: f32, measurement: f32, time: f32) -> (TunerState, f32) {
        match self.phase {
            MotionPhase::Idle => {
                // Initialize step response
                self.initial_measurement = measurement;
                self.start_time = time;
                self.peak_overshoot = measurement;
                self.rise_time_90 = 0.0;
                self.phase = MotionPhase::InjectingStep;
                self.state = TunerState::Running;

                // Output maximum excitation to force a step response
                (self.state, self.max_excitation)
            }
            MotionPhase::InjectingStep => {
                let elapsed = time - self.start_time;
                let current_error = (setpoint - measurement).abs();
                let target_reached = 0.9 * self.target_step;

                // Track peak overshoot
                if measurement > self.peak_overshoot {
                    self.peak_overshoot = measurement;
                }

                // Record time to reach 90% of the step (Rise Time)
                if self.rise_time_90 == 0.0 && (measurement - self.initial_measurement) >= target_reached {
                    self.rise_time_90 = elapsed;
                }

                // If we've been running for too long without settling, or if we've
                // passed the settling time threshold, move to analysis.
                // (Assuming a max tuning timeout of 5.0 seconds for safety)
                if elapsed > 5.0 || (self.rise_time_90 > 0.0 && elapsed > self.rise_time_90 * 3.0) {
                    self.phase = MotionPhase::Analyzing;
                    return (self.state, 0.0); // Stop excitation
                }

                // Continue excitation until settled
                (self.state, self.max_excitation)
            }
            MotionPhase::Analyzing => {
                // Calculate system parameters based on Step Response
                let rise_time = if self.rise_time_90 > 0.001 { self.rise_time_90 } else { 0.1 };
                let overshoot_ratio = (self.peak_overshoot - self.initial_measurement - self.target_step) / self.target_step;

                // Estimate equivalent time constant (T) and gain (K)
                // Simplified first-order plus dead-time (FOPDT) approximation
                let t_eq = rise_time / 2.3;

                // Symmetrical Optimum (SO) tuning rules for motion control
                // Provides excellent disturbance rejection and minimal overshoot.
                // Kp = 1 / (2 * K * t_eq)
                // Ki = 1 / (4 * t_eq^2)
                // Kd = t_eq / 2.0 (Optional, often set to 0 in pure velocity loops)

                let k_est = self.target_step / self.max_excitation; // Crude gain estimate

                self.kp = (1.0 / (2.0 * k_est * t_eq)).max(0.001);
                self.ki = (1.0 / (4.0 * t_eq * t_eq)).max(0.0001);
                self.kd = (t_eq / 2.0).max(0.0);

                // Apply damping if overshoot was severe
                if overshoot_ratio > 0.1 {
                    self.kd *= 1.5;
                    self.kp *= 0.8;
                }

                self.state = TunerState::Complete;
                (self.state, 0.0)
            }
        }
    }

    fn get_gains(&self) -> Option<(f32, f32, f32)> {
        if self.state == TunerState::Complete {
            Some((self.kp, self.ki, self.kd))
        } else {
            None
        }
    }

    fn reset(&mut self) {
        self.state = TunerState::Idle;
        self.phase = MotionPhase::Idle;
        self.kp = 0.0;
        self.ki = 0.0;
        self.kd = 0.0;
    }
}