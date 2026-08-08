
pub struct DynamicLaserPower {
    commanded_power: f32,
    commanded_feedrate: f32,
}

impl DynamicLaserPower {
    pub fn new(commanded_power: f32, commanded_feedrate: f32) -> Self {
        Self {
            commanded_power,
            commanded_feedrate,
        }
    }

    pub fn calculate_pwm(&self, instantaneous_feedrate: f32) -> f32 {
        if self.commanded_feedrate > 0.0 {
            (self.commanded_power / self.commanded_feedrate) * instantaneous_feedrate
        } else {
            0.0
        }
    }
}
