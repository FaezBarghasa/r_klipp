pub struct Pid {
    pub kp: f64,
    pub ki: f64,
    pub kd: f64,
    pub setpoint: f64,
    pub out_min: f64,
    pub out_max: f64,
    integral: f64,
    prev_error: f64,
}

impl Pid {
    pub fn new(kp: f64, ki: f64, kd: f64, setpoint: f64, out_min: f64, out_max: f64) -> Self {
        Self {
            kp,
            ki,
            kd,
            setpoint,
            out_min,
            out_max,
            integral: 0.0,
            prev_error: 0.0,
        }
    }

    pub fn update(&mut self, measurement: f64, dt: f64) -> f64 {
        if dt <= 0.0 {
            return 0.0;
        }
        let error = self.setpoint - measurement;
        self.integral += error * dt;
        
        let p = self.kp * error;
        let i = self.ki * self.integral;
        let d = -self.kd * (measurement - self.prev_error) / dt;
        self.prev_error = measurement;

        let mut out = p + i + d;
        if out > self.out_max {
            out = self.out_max;
            self.integral -= error * dt;
        } else if out < self.out_min {
            out = self.out_min;
            self.integral -= error * dt;
        }
        out
    }
}

pub struct LegacyPid {
    pub proportional: f32,
    pub integral: f32,
    pub derivative: f32,
}