#![no_std]

/// Defines the physical domain being tuned, as Thermal and Mechanical
/// systems require entirely different excitation and analysis methods.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TuningDomain {
    /// Slow, high-inertia system (e.g., Hotends, Enclosures). Uses Relay Feedback.
    Temperature,
    /// Fast, double-integrator system (e.g., Stepper/Servo Axes). Uses Step Response.
    Motion,
}

/// The internal state of the auto-tuning process.
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TunerState {
    Idle,
    Running,
    Complete,
    Failed,
}

/// A unified trait for all auto-tuning algorithms.
pub trait AutoTuner {
    /// Advances the tuner by one step.
    ///
    /// # Arguments
    /// * `setpoint` - The target value.
    /// * `measurement` - The current sensor feedback.
    /// * `time` - Current system time in seconds.
    ///
    /// # Returns
    /// A tuple `(state, output)`:
    /// * `state` - The current status of the tuner.
    /// * `output` - The excitation signal to send to the actuator
    ///              (e.g., Relay ON/OFF for temp, or Max Current/Accel for motion).
    fn update(&mut self, setpoint: f32, measurement: f32, time: f32) -> (TunerState, f32);

    /// Calculates and returns the final PID gains `(kp, ki, kd)`
    /// if the tuning process has successfully completed.
    fn get_gains(&self) -> Option<(f32, f32, f32)>;

    /// Resets the tuner to its initial state.
    fn reset(&mut self);
}