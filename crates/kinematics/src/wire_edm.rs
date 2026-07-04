use crate::{Kinematics, KinematicsError};

pub struct WireEdmKinematics {
    wire_kerf: f32,
}

impl WireEdmKinematics {
    pub fn new(wire_kerf: f32) -> Self {
        Self { wire_kerf }
    }
}

impl Kinematics<4, 4> for WireEdmKinematics {
    fn forward(&self, joints: &[f32; 4]) -> Result<[f32; 4], KinematicsError> {
        Err(KinematicsError::Unreachable)
    }

    fn inverse(&self, cartesian: &[f32; 4]) -> Result<[f32; 4], KinematicsError> {
        let x = cartesian[0];
        let y = cartesian[1];
        let u = cartesian[2];
        let v = cartesian[3];

        // Simplified taper kinematics for now.
        // A full implementation would involve more complex geometry.
        let dx = u - x;
        let dy = v - y;
        let len = micromath::F32Ext::sqrt(dx * dx + dy * dy + 1.0);

        let nx = -dy / len;
        let ny = dx / len;

        let offset_x = nx * self.wire_kerf;
        let offset_y = ny * self.wire_kerf;

        Ok([x - offset_x, y - offset_y, u - offset_x, v - offset_y])
    }
}
