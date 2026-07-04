#![no_std]
use libm;
use crate::tuner_trait::{AutoTuner, TunerState, TuningDomain};
use crate::upgraded_tuner::UpRelayAutotuner;
use crate::motion_tuner::MotionStepTuner;

/// The operational mode of the Smart Controller.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum ControlMode {
    /// Normal closed-loop PID control.
    Run,
    /// Internal Auto-Tuning mode. The controller will output excitation
    /// signals and automatically update its own PID gains upon completion.
    AutoTune,
}

/// A unified, inner-loop auto-tuning controller.
///
/// This struct seamlessly integrates a PID controller with the appropriate
/// auto-tuning algorithm based on the physical domain (Temperature or Motion).
/// It eliminates the need for external state machines to manage the tuning process.
pub struct SmartController {
    // Domain configuration
    domain: TuningDomain,
    mode: ControlMode,

    // PID State (Unified for both domains)
    kp: f32, ki: f32, kd: f32,
    setpoint: f32,
    integral: f32,
    previous_error: f32,
    previous_time: f32,
    output_min: f32,
    output_max: f32,

    // Tuners (Only one is active based on `domain`)
    temp_tuner: UpRelayAutotuner,
    motion_tuner: MotionStepTuner,
}

impl SmartController {
    /// Creates a new SmartController for a specific physical domain.
    pub fn new(domain: TuningDomain, initial_kp: f32, initial_ki: f32, initial_kd: f32) -> Self {
        let (out_min, out_max) = match domain {
            TuningDomain::Temperature => (0.0, 100.0), // PWM %
            TuningDomain::Motion => (-100.0, 100.0),   // Current/Torque %
        };

        Self {
            domain,
            mode: ControlMode::Run,
            kp: initial_kp, ki: initial_ki, kd: initial_kd,
            setpoint: 0.0,
            integral: 0.0,
            previous_error: 0.0,
            previous_time: 0.0,
            output_min: out_min,
            output_max: out_max,
            temp_tuner: UpRelayAutotuner::new(0.0, 2.0, 100.0, crate::upgraded_tuner::TuneMode::Heating),
            motion_tuner: MotionStepTuner::new(100.0, 50.0), // Max accel, step size
        }
    }

    /// Commands the controller to start the internal auto-tuning sequence.
    pub fn start_autotune(&mut self, setpoint: f32) {
        self.setpoint = setpoint;
        self.mode = ControlMode::AutoTune;
        self.integral = 0.0;
        self.previous_error = 0.0;

        match self.domain {
            TuningDomain::Temperature => {
                self.temp_tuner = UpRelayAutotuner::new(setpoint, 2.0, self.output_max, crate::upgraded_tuner::TuneMode::Heating);
            }
            TuningDomain::Motion => {
                // For motion, the setpoint acts as the target velocity/position step
                self.motion_tuner = MotionStepTuner::new(self.output_max, setpoint);
            }
        }
    }

    /// The core inner-loop update function.
    ///
    /// If in `Run` mode, it computes standard PID.
    /// If in `AutoTune` mode, it drives the tuner, outputs the excitation signal,
    /// and automatically applies the new PID gains when the tuner finishes.
    pub fn compute(&mut self, measurement: f32, time: f32) -> f32 {
        match self.mode {
            ControlMode::Run => self.compute_pid(measurement, time),
            ControlMode::AutoTune => self.compute_tune(measurement, time),
        }
    }

    /// Internal PID computation with domain-specific filtering.
    fn compute_pid(&mut self, measurement: f32, time: f32) -> f32 {
        let dt = time - self.previous_time;
        if dt <= 0.0001 { return 0.0; } // Prevent division by zero

        let error = self.setpoint - measurement;

        // Proportional
        let p_term = self.kp * error;

        // Integral with Anti-Windup
        self.integral += error * dt;
        let i_max = self.output_max / if self.ki.abs() > 0.0001 { self.ki } else { 0.0001 };
        let i_min = self.output_min / if self.ki.abs() > 0.0001 { self.ki } else { 0.0001 };

        // Clamp integral based on domain (Temp only integrates positive, Motion can be negative)
        if self.domain == TuningDomain::Temperature {
            self.integral = self.integral.max(0.0).min(i_max);
        } else {
            self.integral = self.integral.max(i_min).min(i_max);
        }
        let i_term = self.ki * self.integral;

        // Derivative on Measurement (avoids derivative kick on setpoint changes)
        // Crucial for motion systems to prevent violent jerks when target changes.
        let d_term = -self.kd * (measurement - self.previous_error) / dt;

        let mut output = p_term + i_term + d_term;

        // Output Clamping & Anti-Windup Back-calculation
        if output > self.output_max {
            output = self.output_max;
            if self.ki > 0.0 { self.integral -= error * dt; } // Prevent windup
        } else if output < self.output_min {
            output = self.output_min;
            if self.ki > 0.0 { self.integral -= error * dt; }
        }

        self.previous_error = measurement;
        self.previous_time = time;

        output
    }

    /// Internal Auto-Tune computation.
    fn compute_tune(&mut self, measurement: f32, time: f32) -> f32 {
        let (state, excitation) = match self.domain {
            TuningDomain::Temperature => {
                self.temp_tuner.update(self.setpoint, measurement, time)
            }
            TuningDomain::Motion => {
                self.motion_tuner.update(self.setpoint, measurement, time)
            }
        };

        // Check if tuning completed successfully
        if state == TunerState::Complete {
            if let Some((new_kp, new_ki, new_kd)) = self.get_current_tuner_gains() {
                self.kp = new_kp;
                self.ki = new_ki;
                self.kd = new_kd;
            }
            self.mode = ControlMode::Run; // Automatically revert to Run mode
            self.integral = 0.0; // Reset integral for the new gains
        } else if state == TunerState::Failed {
            self.mode = ControlMode::Run; // Abort tuning on failure
        }

        excitation
    }

    /// Helper to extract gains from the active tuner.
    fn get_current_tuner_gains(&self) -> Option<(f32, f32, f32)> {
        match self.domain {
            TuningDomain::Temperature => self.temp_tuner.get_tunings(),
            TuningDomain::Motion => self.motion_tuner.get_gains(),
        }
    }

    // --- Standard Setters ---
    pub fn set_setpoint(&mut self, setpoint: f32) { self.setpoint = setpoint; }
    pub fn set_output_limits(&mut self, min: f32, max: f32) { self.output_min = min; self.output_max = max; }
    pub fn get_mode(&self) -> ControlMode { self.mode }
    pub fn get_gains(&self) -> (f32, f32, f32) { (self.kp, self.ki, self.kd) }
}