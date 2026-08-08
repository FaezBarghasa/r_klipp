//! Advanced PID controller for motion systems.
//! This corresponds to Task 3.1.

#![cfg_attr(not(test), no_std)]

use micromath::F32Ext;

#[derive(Debug, Clone, Copy)]
pub struct PidGains {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    pub kv: f32, // Velocity feed-forward
    pub ka: f32, // Acceleration feed-forward
}

#[derive(Debug, Clone, Copy)]
pub struct PidController {
    gains: PidGains,
    integral: f32,
    previous_error: f32,
    previous_measurement: f32,
    previous_time: f32,
    derivative: f32,
    output_limit_min: f32,
    output_limit_max: f32,
}

impl PidController {
    pub fn new(gains: PidGains, output_limit_min: f32, output_limit_max: f32) -> Self {
        Self {
            gains,
            integral: 0.0,
            previous_error: 0.0,
            previous_measurement: 0.0,
            previous_time: 0.0,
            derivative: 0.0,
            output_limit_min,
            output_limit_max,
        }
    }

    pub fn reset(&mut self) {
        self.integral = 0.0;
        self.previous_error = 0.0;
        self.previous_measurement = 0.0;
        self.previous_time = 0.0;
        self.derivative = 0.0;
    }

    pub fn update(&mut self, setpoint: f32, measurement: f32, time: f32, velocity_cmd: f32, accel_cmd: f32) -> f32 {
        let dt = if self.previous_time == 0.0 {
            0.0
        } else {
            time - self.previous_time
        };
        self.previous_time = time;

        if dt <= 0.0 {
            // If dt is not positive, we can't do any calculations.
            // Return the feed-forward terms only.
            return self.gains.kv * velocity_cmd + self.gains.ka * accel_cmd;
        }

        let error = setpoint - measurement;

        // Proportional term
        let p_term = self.gains.kp * error;

        // Integral term with back-calculation anti-windup
        let i_term = self.integral;

        // Derivative on measurement with low-pass filter
        let measurement_deriv = (measurement - self.previous_measurement) / dt;
        self.previous_measurement = measurement;
        // Simple low-pass filter (alpha can be tuned)
        let alpha = 0.5;
        self.derivative = (1.0 - alpha) * self.derivative + alpha * measurement_deriv;
        let d_term = self.gains.kd * self.derivative;

        // Feed-forward terms
        let ff_term = self.gains.kv * velocity_cmd + self.gains.ka * accel_cmd;

        // Combine terms and clamp output
        let mut output = p_term + i_term - d_term + ff_term;
        let clamped_output = output.clamp(self.output_limit_min, self.output_limit_max);

        // Anti-windup logic
        let anti_windup_gain = 0.1; // Tunable gain
        self.integral += self.gains.ki * error * dt + anti_windup_gain * (clamped_output - output) * dt;

        self.previous_error = error;

        clamped_output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use approx::assert_relative_eq;

    #[test]
    fn test_pid_dt_calculation() {
        let gains = PidGains { kp: 1.0, ki: 0.1, kd: 0.01, kv: 0.0, ka: 0.0 };
        let mut pid = PidController::new(gains, -100.0, 100.0);

        // First update, dt should be 0
        pid.update(10.0, 0.0, 0.0, 0.0, 0.0);
        assert_eq!(pid.previous_time, 0.0);

        // Second update
        pid.update(10.0, 1.0, 0.001, 0.0, 0.0);
        assert_eq!(pid.previous_time, 0.001);
        // dt is 0.001, so integral should have changed
        assert_ne!(pid.integral, 0.0);
    }

    #[test]
    fn test_derivative_on_measurement_kick() {
        let gains = PidGains { kp: 1.0, ki: 0.0, kd: 10.0, kv: 0.0, ka: 0.0 };
        let mut pid = PidController::new(gains, -100.0, 100.0);

        // Initial state
        pid.update(0.0, 0.0, 0.0, 0.0, 0.0);
        let output1 = pid.update(0.0, 0.0, 0.1, 0.0, 0.0);
        assert_relative_eq!(output1, 0.0, epsilon = 1e-6);

        // Step change in setpoint - should NOT cause a derivative kick
        let output2 = pid.update(10.0, 0.0, 0.2, 0.0, 0.0);
        // P-term is 1.0 * 10.0 = 10.0. D-term should be zero.
        assert_relative_eq!(output2, 10.0, epsilon = 1e-6);
    }

    #[test]
    fn test_feed_forward() {
        let gains = PidGains { kp: 0.0, ki: 0.0, kd: 0.0, kv: 1.5, ka: 0.5 };
        let mut pid = PidController::new(gains, -100.0, 100.0);

        let output = pid.update(10.0, 10.0, 1.0, 10.0, 2.0);
        // With zero error, output should be only the FF terms
        // 1.5 * 10.0 (kv * vel) + 0.5 * 2.0 (ka * acc) = 15.0 + 1.0 = 16.0
        assert_relative_eq!(output, 16.0);
    }
}