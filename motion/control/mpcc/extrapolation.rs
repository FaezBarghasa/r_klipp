//! Polynomial extrapolation for servo interpolation.

pub struct Extrapolator {
    // Previous state [pos, vel, acc]
    p0: [f32; 3],
    p1: [f32; 3],
    dt_plc: f32,
}

impl Extrapolator {
    pub fn new(dt_plc: f32) -> Self {
        Self {
            p0: [0.0; 3],
            p1: [0.0; 3],
            dt_plc,
        }
    }

    pub fn update_state(&mut self, pos: f32, vel: f32, acc: f32) {
        self.p0 = self.p1;
        self.p1 = [pos, vel, acc];
    }

    pub fn extrapolate(&self, dt_servo: f32) -> (f32, f32, f32) {
        let t = dt_servo / self.dt_plc;
        let pos = self.p1[0] + self.p1[1] * t + 0.5 * self.p1[2] * t * t;
        let vel = self.p1[1] + self.p1[2] * t;
        let acc = self.p1[2];
        (pos, vel, acc)
    }
}
