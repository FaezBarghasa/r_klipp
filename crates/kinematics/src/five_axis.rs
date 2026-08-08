use crate::matrix::Matrix3x3;
use crate::{Kinematics, KinematicsError};

pub struct FiveAxisKinematics {
    tool_vector: [f32; 3],
    pivot_length: f32,
}

impl FiveAxisKinematics {
    pub fn new(tool_vector: [f32; 3], pivot_length: f32) -> Self {
        Self {
            tool_vector,
            pivot_length,
        }
    }
}

impl Kinematics<5, 5> for FiveAxisKinematics {
    fn forward(&self, _joints: &[f32; 5]) -> Result<[f32; 5], KinematicsError> {
        // Forward kinematics for 5-axis is more complex, so we'll leave it for later.
        Err(KinematicsError::Unreachable)
    }

    fn inverse(&self, cartesian: &[f32; 5]) -> Result<[f32; 5], KinematicsError> {
        let programmed_linear = [cartesian[0], cartesian[1], cartesian[2]];
        let rotary_axes = [cartesian[3], cartesian[4], 0.0]; // Assuming A, B axes

        let rotation_matrix = Matrix3x3::from_euler_angles(rotary_axes[0], rotary_axes[1], rotary_axes[2]);
        let tool_offset = rotation_matrix * self.tool_vector;

        let compensated_linear = [
            programmed_linear[0] - tool_offset[0] * self.pivot_length,
            programmed_linear[1] - tool_offset[1] * self.pivot_length,
            programmed_linear[2] - tool_offset[2] * self.pivot_length,
        ];

        Ok([
            compensated_linear[0],
            compensated_linear[1],
            compensated_linear[2],
            rotary_axes[0],
            rotary_axes[1],
        ])
    }
}
