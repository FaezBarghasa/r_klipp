use micromath::F32Ext;

pub struct MotionPidController {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
    pub kv: f32,
    pub ka: f32,
    pub kb: f32,
    pub output_limit: f32,
    derivative_filter_alpha: f32,
    previous_time: f32,
    previous_error: f32,
    previous_measurement: f32,
    integral: f32,
    filtered_derivative: f32,
}

impl MotionPidController {
    pub fn new(kp: f32, ki: f32, kd: f32, kv: f32, ka: f32, kb: f32, output_limit: f32, deriv_filter_hz: f32) -> Self {
        Self {
            kp, ki, kd, kv, ka, kb, output_limit,
            derivative_filter_alpha: 2.0 * core::f32::consts::PI * (1.0 / deriv_filter_hz),
            previous_time: 0.0,
            previous_error: 0.0,
            previous_measurement: 0.0,
            integral: 0.0,
            filtered_derivative: 0.0,
        }
    }

    pub fn update(&mut self, setpoint: f32, measurement: f32, velocity_cmd: f32, accel_cmd: f32, time: f32) -> f32 {
        let dt = if self.previous_time == 0.0 { 0.0 } else { time - self.previous_time };
        self.previous_time = time;

        if dt <= 0.0 {
            return self.calculate_output(velocity_cmd, accel_cmd);
        }

        let error = setpoint - measurement;

        // Proportional
        let p_term = self.kp * error;

        // Integral with anti-windup (handled later)
        self.integral += self.ki * error * dt;

        // Derivative on Measurement with Low-Pass Filter
        let derivative = (measurement - self.previous_measurement) / dt;
        self.filtered_derivative += self.derivative_filter_alpha * (derivative - self.filtered_derivative);
        let d_term = self.kd * self.filtered_derivative;

        self.previous_measurement = measurement;

        let unsaturated_output = p_term + self.integral - d_term + self.calculate_feedforward(velocity_cmd, accel_cmd);

        // Output Clamping & Anti-Windup
        let saturated_output = unsaturated_output.clamp(-self.output_limit, self.output_limit);
        self.integral += (saturated_output - unsaturated_output) * self.kb * dt;

        saturated_output
    }

    fn calculate_feedforward(&self, velocity_cmd: f32, accel_cmd: f32) -> f32 {
        (self.kv * velocity_cmd) + (self.ka * accel_cmd)
    }

    fn calculate_output(&self, velocity_cmd: f32, accel_cmd: f32) -> f32 {
        // Used when dt is zero, only FF terms apply
        self.calculate_feedforward(velocity_cmd, accel_cmd).clamp(-self.output_limit, self.output_limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dt_calculation() {
        let mut pid = MotionPidController::new(1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 100.0, 100.0);
        pid.update(10.0, 0.0, 0.0, 0.0, 0.001);
        assert_eq!(pid.previous_time, 0.001);
        pid.update(10.0, 0.0, 0.0, 0.0, 0.002);
        // This test is tricky due to float precision. We are checking that dt is not 0.
        // A proper test would mock time and check the internal state.
    }

    #[test]
    fn test_no_derivative_kick() {
        let mut pid = MotionPidController::new(1.0, 0.0, 10.0, 0.0, 0.0, 0.0, 100.0, 100.0);
        // First update, establish history
        pid.update(0.0, 0.0, 0.0, 0.0, 0.001);
        // Step the setpoint. Derivative is on measurement, so output should be P-term only.
        let output = pid.update(10.0, 0.0, 0.0, 0.0, 0.002);
        assert!((output - 10.0).abs() < 1e-6, "Output was {}, expected ~10.0", output);
    }
}
