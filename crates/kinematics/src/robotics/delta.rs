use nalgebra::{Vector3, Matrix3};
use libm::{sqrtf, powf};

const E: f32 = 100.0; // End effector side length
const F: f32 = 200.0; // Base side length
const RE: f32 = 150.0; // End effector arm length
const RF: f32 = 50.0;  // Base arm length

pub struct Delta {
    // Geometric parameters
}

impl Delta {
    // Forward kinematics is complex and often not needed in the controller
    pub fn forward_kinematics(&self, _joint_angles: &[f32; 3]) -> Result<Vector3<f32>, &'static str> {
        Err("Forward kinematics for Delta is non-trivial and not implemented here")
    }

    // Inverse kinematics for a Delta robot
    pub fn inverse_kinematics(&self, target_pos: &Vector3<f32>) -> Result<[f32; 3], &'static str> {
        let mut joint_angles = [0.0; 3];
        let (x0, y0, z0) = (target_pos.x, target_pos.y, target_pos.z);

        for i in 0..3 {
            let angle_rad = (i as f32 * 120.0).to_radians();
            let x = x0 * angle_rad.cos() + y0 * angle_rad.sin();
            let y = y0 * angle_rad.cos() - x0 * angle_rad.sin();

            let j1_y = -(F - E) / (2.0 * sqrtf(3.0));
            let e1_y = (F - E) / (2.0 * sqrtf(3.0)) + y;

            let w_sq = powf(RF, 2.0);
            let l_sq = powf(RE, 2.0);
            let a = (x - RF).powi(2) + e1_y.powi(2) + z0.powi(2);
            let b = 2.0 * z0 * (j1_y - e1_y);
            let c = (j1_y - e1_y).powi(2) - w_sq + l_sq;

            let discriminant = b.powi(2) - 4.0 * a * c;
            if discriminant < 0.0 {
                return Err("Target out of workspace");
            }

            let t = (-b - sqrtf(discriminant)) / (2.0 * a);
            joint_angles[i] = (2.0 * t).atan();
        }

        Ok(joint_angles)
    }
}
