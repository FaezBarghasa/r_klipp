use micromath::F32Ext;

pub struct ThermalPidController {
    pub heat_kp: f32,
    pub heat_ki: f32,
    pub heat_kd: f32,
    pub hysteresis: f32,
    pub output_limit: f32,
    previous_time: f32,
    integral: f32,
    previous_error: f32,
    is_heating: bool,
}

impl ThermalPidController {
    pub fn new(heat_gains: (f32, f32, f32), hysteresis: f32, output_limit: f32) -> Self {
        Self {
            heat_kp: heat_gains.0, heat_ki: heat_gains.1, heat_kd: heat_gains.2,

            hysteresis, output_limit,
            previous_time: 0.0,
            integral: 0.0,
            previous_error: 0.0,
            is_heating: true,
        }
    }

    pub fn update(&mut self, setpoint: f32, measurement: f32, time: f32) -> f32 {
        let dt = if self.previous_time == 0.0 { 0.0 } else { time - self.previous_time };
        self.previous_time = time;

        if dt <= 0.0 { return 0.0; }

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

        // Integral with anti-windup
        self.integral += ki * error * dt;
        self.integral = self.integral.clamp(-self.output_limit, self.output_limit);

        // Derivative on Error (acceptable for thermal systems)
        let derivative = (error - self.previous_error) / dt;
        let d_term = kd * derivative;

        self.previous_error = error;

        let output = p_term + self.integral + d_term;
        output.clamp(-self.output_limit, self.output_limit)
    }
}
