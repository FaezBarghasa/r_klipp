//! A PID (Proportional-Integral-Derivative) controller.

use num_traits::{Float, Signed};

/// A PID controller for regulating a system.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Pid<F: Float + Signed> {
    /// Proportional gain.
    pub kp: F,
    /// Integral gain.
    pub ki: F,
    /// Derivative gain.
    pub kd: F,

    /// The target value for the system.
    pub setpoint: F,

    /// The lower bound of the controller's output.
    pub output_min: F,
    /// The upper bound of the controller's output.
    pub output_max: F,

    // Internal state
    integral: F,
    previous_error: F,
}

impl<F: Float + Signed> Pid<F> {
    /// Creates a new PID controller.
    pub fn new(kp: F, ki: F, kd: F, setpoint: F, output_min: F, output_max: F) -> Self {
        Self {
            kp,
            ki,
            kd,
            setpoint,
            output_min,
            output_max,
            integral: F::zero(),
            previous_error: F::zero(),
        }
    }

    /// Updates the PID controller with a new measurement.
    ///
    /// # Arguments
    /// * `current_value` - The current measured value of the system.
    /// * `dt` - The time delta since the last update, in seconds.
    ///
    /// # Returns
    /// The calculated output value for the actuator.
    pub fn update(&mut self, current_value: F, dt: F) -> F {
        let error = self.setpoint - current_value;

        // Proportional term
        let p_term = self.kp * error;

        // Integral term with anti-windup
        self.integral = self.integral + error * dt;
        // Clamp the integral term to prevent windup
        let integral_limit = self.output_max / self.ki.max(F::from(0.001).unwrap());
        self.integral = self.integral.max(-integral_limit).min(integral_limit);

        let i_term = self.ki * self.integral;

        // Derivative term
        let derivative = (error - self.previous_error) / dt;
        let d_term = self.kd * derivative;

        // Update state for next iteration
        self.previous_error = error;

        // Calculate final output
        let mut output = p_term + i_term + d_term;

        // Clamp output to the defined limits
        output = output.max(self.output_min).min(self.output_max);

        output
    }

    /// Resets the internal state of the PID controller.
    pub fn reset(&mut self) {
        self.integral = F::zero();
        self.previous_error = F::zero();
    }
}
