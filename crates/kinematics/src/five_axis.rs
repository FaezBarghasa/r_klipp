use crate::matrix::Matrix3x3;
use crate::{Kinematics, KinematicsError};

pub struct FiveAxisKinematics {
    tool_vector: [f64; 3],
    pivot_length: f64,
}

impl FiveAxisKinematics {
    pub fn new(tool_vector: [f64; 3], pivot_length: f64) -> Self {
        Self {
            tool_vector,
            pivot_length,
        }
    }
}

impl Kinematics for FiveAxisKinematics {
    fn cartesian_to_motors(&self, cartesian: [f64; 4]) -> Result<[f64; 4], KinematicsError> {
        let programmed_linear = [cartesian[0], cartesian[1], cartesian[2]];
        let rotary_a = cartesian[3]; // A-axis rotation in radians

        let rotation_matrix = Matrix3x3::from_euler_angles(rotary_a as f32, 0.0, 0.0);
        let tool_vec_f32 = [self.tool_vector[0] as f32, self.tool_vector[1] as f32, self.tool_vector[2] as f32];
        let tool_offset = rotation_matrix * tool_vec_f32;

        let compensated_linear = [
            programmed_linear[0] - tool_offset[0] as f64 * self.pivot_length,
            programmed_linear[1] - tool_offset[1] as f64 * self.pivot_length,
            programmed_linear[2] - tool_offset[2] as f64 * self.pivot_length,
        ];

        Ok([
            compensated_linear[0],
            compensated_linear[1],
            compensated_linear[2],
            rotary_a,
        ])
    }

    fn motors_to_cartesian(&self, motors: [f64; 4]) -> Result<[f64; 4], KinematicsError> {
        Ok(motors)
    }
}
