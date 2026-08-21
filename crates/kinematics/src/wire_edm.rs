use crate::{Kinematics, KinematicsError};

pub struct WireEdmKinematics {
    pub wire_kerf: f64,
}

impl WireEdmKinematics {
    pub fn new(wire_kerf: f64) -> Self {
        Self { wire_kerf }
    }
}

impl Kinematics for WireEdmKinematics {
    fn cartesian_to_motors(&self, cartesian: [f64; 4]) -> Result<[f64; 4], KinematicsError> {
        let x = cartesian[0];
        let y = cartesian[1];
        let u = cartesian[2];
        let v = cartesian[3];

        let dx = u - x;
        let dy = v - y;
        let len = (dx * dx + dy * dy + 1.0).sqrt();

        let nx = -dy / len;
        let ny = dx / len;

        let offset_x = nx * self.wire_kerf;
        let offset_y = ny * self.wire_kerf;

        Ok([x - offset_x, y - offset_y, u - offset_x, v - offset_y])
    }

    fn motors_to_cartesian(&self, motors: [f64; 4]) -> Result<[f64; 4], KinematicsError> {
        Ok(motors)
    }
}
