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
    previous_measurement: f32,
    integral: f32,
    filtered_derivative: f32,
}

impl MotionPidController {
    pub fn new(kp: f32, ki: f32, kd: f32, kv: f32, ka: f32, kb: f32, output_limit: f32, deriv_filter_hz: f32, sample_rate_hz: f32) -> Self {
        let dt = 1.0 / sample_rate_hz;
        let rc = 1.0 / (2.0 * core::f32::consts::PI * deriv_filter_hz);
        let alpha = dt / (rc + dt);
        Self {
            kp, ki, kd, kv, ka, kb, output_limit,
            derivative_filter_alpha: alpha,
            previous_time: 0.0,
            previous_measurement: 0.0,
            integral: 0.0,
            filtered_derivative: 0.0,
        }
    }

    pub fn update(&mut self, setpoint: f32, measurement: f32, velocity_cmd: f32, accel_cmd: f32, time: f32) -> f32 {
        let dt = if self.previous_time == 0.0 { 0.0 } else { time - self.previous_time };
        self.previous_time = time;

        let ff_term = (self.kv * velocity_cmd) + (self.ka * accel_cmd);

        if dt <= 0.0 {
            return ff_term.clamp(-self.output_limit, self.output_limit);
        }

        let error = setpoint - measurement;

        // Proportional
        let p_term = self.kp * error;

        // Integral
        let i_term = self.integral;

        // Derivative on Measurement with Low-Pass Filter
        let derivative = (measurement - self.previous_measurement) / dt;
        self.filtered_derivative = self.derivative_filter_alpha * derivative + (1.0 - self.derivative_filter_alpha) * self.filtered_derivative;
        let d_term = self.kd * self.filtered_derivative;

        self.previous_measurement = measurement;

        let unsaturated_output = p_term + i_term - d_term + ff_term;

        // Output Clamping & Back-Calculation Anti-Windup
        let saturated_output = unsaturated_output.clamp(-self.output_limit, self.output_limit);
        self.integral += self.ki * error * dt + (saturated_output - unsaturated_output) * self.kb;

        saturated_output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dt_is_correct() {
        let mut pid = MotionPidController::new(1.0, 1.0, 1.0, 0.0, 0.0, 0.0, 100.0, 100.0, 1000.0);
        pid.update(10.0, 0.0, 0.0, 0.0, 0.001);
        // After first update, dt is 0, so only ff is returned.
        let output = pid.update(10.0, 0.0, 0.0, 0.0, 0.002);
        // dt = 0.001. error = 10. p_term = 10. integral becomes 1*10*0.001=0.01.
        // unsaturated = 10 + 0.01 = 10.01.
        // previous_measurement was 0, now 0. derivative is 0.
        // Check that integral term is behaving as expected.
        assert!((output - 10.01).abs() < 1e-6, "Output was {}, expected ~10.01", output);
    }

    #[test]
    fn test_no_derivative_kick_on_setpoint_change() {
        let mut pid = MotionPidController::new(1.0, 0.0, 10.0, 0.0, 0.0, 0.0, 100.0, 100.0, 1000.0);
        // First update, establish history
        pid.update(0.0, 0.0, 0.0, 0.0, 0.001);
        // Step the setpoint. Derivative is on measurement, so output should be P-term only.
        let output = pid.update(10.0, 0.0, 0.0, 0.0, 0.002);
        assert!((output - 10.0).abs() < 1e-6, "Output was {}, expected ~10.0", output);
    }

    #[test]
    fn test_feedforward_terms() {
        let mut pid = MotionPidController::new(0.0, 0.0, 0.0, 2.0, 3.0, 0.0, 100.0, 100.0, 1000.0);
        let output = pid.update(10.0, 10.0, 5.0, 1.0, 0.001);
        // With zero error, output should be only the feedforward terms
        // kv*vel + ka*accel = 2*5 + 3*1 = 13
        assert!((output - 13.0).abs() < 1e-6);
    }
}