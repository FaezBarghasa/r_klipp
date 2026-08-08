use micromath::F32Ext;

pub struct ThermalPidController {
    pub heat_kp: f32,
    pub heat_ki: f32,
    pub heat_kd: f32,
    pub cool_kp: f32,
    pub cool_ki: f32,
    pub cool_kd: f32,
    pub hysteresis: f32,
    pub output_limit_min: f32,
    pub output_limit_max: f32,
    previous_time: f32,
    integral: f32,
    previous_error: f32,
    is_heating: bool,
}

impl ThermalPidController {
    pub fn new(
        heat_gains: (f32, f32, f32),
        cool_gains: (f32, f32, f32),
        hysteresis: f32,
        output_limits: (f32, f32),
    ) -> Self {
        Self {
            heat_kp: heat_gains.0,
            heat_ki: heat_gains.1,
            heat_kd: heat_gains.2,
            cool_kp: cool_gains.0,
            cool_ki: cool_gains.1,
            cool_kd: cool_gains.2,
            hysteresis,
            output_limit_min: output_limits.0,
            output_limit_max: output_limits.1,
            previous_time: 0.0,
            integral: 0.0,
            previous_error: 0.0,
            is_heating: true,
        }
    }

    pub fn update(&mut self, setpoint: f32, measurement: f32, time: f32) -> f32 {
        let dt = if self.previous_time == 0.0 { 0.0 } else { time - self.previous_time };
        self.previous_time = time;

        if dt <= 1e-6 {
            return 0.0;
        }

        let error = setpoint - measurement;

        // Hysteresis for switching between heating and cooling modes
        if self.is_heating && error < -self.hysteresis {
            self.is_heating = false;
            self.integral = 0.0; // Reset integral on mode switch
        } else if !self.is_heating && error > self.hysteresis {
            self.is_heating = true;
            self.integral = 0.0;
        }

        let (kp, ki, kd) = if self.is_heating {
            (self.heat_kp, self.heat_ki, self.heat_kd)
        } else {
            (self.cool_kp, self.cool_ki, self.cool_kd)
        };

        // Proportional
        let p_term = kp * error;

        // Integral
        let i_term = self.integral;

        // Derivative on Error (acceptable for thermal systems)
        let derivative = (error - self.previous_error) / dt;
        let d_term = kd * derivative;

        self.previous_error = error;

        let unsaturated_output = p_term + i_term + d_term;
        let saturated_output = unsaturated_output.clamp(self.output_limit_min, self.output_limit_max);

        // Back-calculation anti-windup
        // For thermal controllers, a simpler integral update is often sufficient.
        // We adjust the integral based on the amount of output saturation.
        // A kb (back-calculation gain) of 1/ki can be a good starting point.
        let kb = 1.0 / ki.max(1e-6);
        self.integral += ki * error * dt + (saturated_output - unsaturated_output) * kb;


        saturated_output
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermal_pid_heating() {
        let mut pid = ThermalPidController::new(
            (10.0, 0.1, 0.5),
            (5.0, 0.05, 0.2),
            2.0,
            (0.0, 100.0),
        );
        let output = pid.update(100.0, 90.0, 0.1);
        assert!(output > 0.0);
    }

    #[test]
    fn test_thermal_pid_switching_to_cooling() {
        let mut pid = ThermalPidController::new(
            (10.0, 0.1, 0.5),
            (5.0, 0.05, 0.2),
            2.0,
            (-100.0, 100.0),
        );
        pid.update(100.0, 95.0, 0.1); // Start in heating mode
        assert!(pid.is_heating);
        // Error is now -3, which is less than -hysteresis (-2)
        pid.update(100.0, 103.0, 0.2);
        assert!(!pid.is_heating);
    }
}