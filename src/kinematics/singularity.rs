
// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

#![allow(non_snake_case)]

use crate::kinematics::poe_fk::PoeKinematics;

impl<const N: usize> PoeKinematics<N> {
    /// Calculates the Yoshikawa Manipulability Measure.
    /// w = sqrt(det(J * J^T))
    pub fn manipulability(&self, joint_angles: &[f64; N]) -> Result<f64, &'static str> {
        if N > 6 {
            // For redundant robots, the concept is slightly different.
            // This implementation is for non-redundant manipulators.
            return Err("Manipulability for redundant robots not implemented.");
        }

        let J = self.jacobian(joint_angles)?;
        let JJ_t = J * J.transpose();
        let det = JJ_t.determinant().ok_or("Failed to calculate determinant")?;

        Ok(libm::sqrt(det.abs()))
    }

    /// Adaptively adjusts the damping factor based on manipulability.
    pub fn adaptive_damping(&self, joint_angles: &[f64; N], lambda_max: f64, w_threshold: f64) -> Result<f64, &'static str> {
        let w = self.manipulability(joint_angles)?;
        if w < w_threshold {
            Ok(lambda_max * (1.0 - (w / w_threshold).powi(2)))
        } else {
            Ok(0.0)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinematics::math::se3::{Transform, Twist};
    use crate::math::{Matrix4, Vector3};

    fn get_test_kinematics() -> PoeKinematics<6> {
        let screw_axes = [
            Twist { angular: Vector3::new(0.0, 0.0, 1.0), linear: Vector3::new(0.0, 0.0, 0.0) },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, 0.0) },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -0.425) },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -0.817) },
            Twist { angular: Vector3::new(0.0, 0.0, -1.0), linear: Vector3::new(0.0, 0.0, 0.0) },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -1.014) },
        ];
        let M = Transform(Matrix4::from_translation(&Vector3::new(0.0, -0.109, 1.014)));
        PoeKinematics { screw_axes, M }
    }

    #[test]
    fn test_manipulability_at_zero() {
        let kinematics = get_test_kinematics();
        let joint_angles = [0.0; 6];
        let w = kinematics.manipulability(&joint_angles).unwrap();
        // A non-zero, non-singular configuration should have manipulability > 0
        assert!(w > 1e-6);
    }

    #[test]
    fn test_adaptive_damping() {
        let kinematics = get_test_kinematics();
        let joint_angles = [0.0; 6]; // Non-singular pose
        let lambda = kinematics.adaptive_damping(&joint_angles, 0.1, 0.01).unwrap();
        assert_eq!(lambda, 0.0);

        // A known singular pose for many 6-axis arms (wrist singularity)
        let singular_angles = [0.0, 0.0, 0.0, 0.0, core::f64::consts::PI, 0.0];
        let w = kinematics.manipulability(&singular_angles).unwrap();
        assert!(w < 1e-6); // Should be very close to zero

        let lambda_singular = kinematics.adaptive_damping(&singular_angles, 0.1, 0.01).unwrap();
        assert!(lambda_singular > 0.09); // Should be close to lambda_max
    }
}
