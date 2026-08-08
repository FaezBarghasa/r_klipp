use nalgebra::{Vector3, Matrix4};
use libm::{cosf, sinf, acosf, atan2f, fabsf};

pub struct Scara {
    link1: f32,
    link2: f32,
}

impl Scara {
    pub fn forward_kinematics(&self, joint_angles: &[f32; 4]) -> Vector3<f32> {
        let (theta1, theta2, d3, _theta4) = (joint_angles[0], joint_angles[1], joint_angles[2], joint_angles[3]);
        let x = self.link1 * cosf(theta1) + self.link2 * cosf(theta1 + theta2);
        let y = self.link1 * sinf(theta1) + self.link2 * sinf(theta1 + theta2);
        let z = d3;
        Vector3::new(x, y, z)
    }

    pub fn inverse_kinematics(&self, target_pos: &Vector3<f32>) -> Result<[f32; 4], &'static str> {
        let (x, y, z) = (target_pos.x, target_pos.y, target_pos.z);
        let d3 = z;

        let cos_theta2 = (x * x + y * y - self.link1 * self.link1 - self.link2 * self.link2) / (2.0 * self.link1 * self.link2);
        if fabsf(cos_theta2) > 1.0 {
            return Err("Target out of reach");
        }
        let theta2 = acosf(cos_theta2); // Elbow up solution

        let k1 = self.link1 + self.link2 * cosf(theta2);
        let k2 = self.link2 * sinf(theta2);
        let theta1 = atan2f(y, x) - atan2f(k2, k1);

        // For a SCARA, the 4th joint is often independent orientation
        let theta4 = 0.0; // Assuming no specific orientation is required

        Ok([theta1, theta2, d3, theta4])
    }
}
