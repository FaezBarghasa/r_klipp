
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

use crate::kinematics::math::se3::Transform;
use crate::kinematics::poe_fk::PoeKinematics;
use crate::math::{Matrix, Vector};

const IK_MAX_ITERATIONS: usize = 4;
const IK_TOLERANCE: f64 = 1e-4;

impl<const N: usize> PoeKinematics<N> {
    /// Solves the inverse kinematics using the Damped Least Squares (Levenberg-Marquardt) method.
    pub fn inverse_dls(
        &self,
        target_transform: &Transform,
        initial_joint_angles: &[f64; N],
        lambda: f64,
    ) -> Result<[f64; N], &'static str> {
        let mut current_angles = *initial_joint_angles;

        for _ in 0..IK_MAX_ITERATIONS {
            let current_transform = self.forward(&current_angles)?;
            let error = Self::calculate_error(target_transform, &current_transform);

            if error.norm() < IK_TOLERANCE {
                return Ok(current_angles);
            }

            let J = self.jacobian(&current_angles)?;
            let J_t = J.transpose();
            let JJ_t = J * J_t;
            let damping = Matrix::<6, 6>::identity() * (lambda * lambda);
            let inv_term = (JJ_t + damping).inverse().ok_or("Matrix inversion failed")?;
            let J_dls = J_t * inv_term;

            let delta_theta: Vector<N> = J_dls * error;

            for i in 0..N {
                current_angles[i] += delta_theta[i];
            }
        }

        Err("IK failed to converge")
    }

    /// Calculates the 6x1 error vector between the target and current transform.
    fn calculate_error(target: &Transform, current: &Transform) -> Vector<6> {
        let error_transform = *target * current.inverse();
        let p_error = error_transform.0.get_translation();
        let R_error = error_transform.0.get_rotation();

        // Use axis-angle representation for orientation error
        let angle = libm::acos((R_error.trace() - 1.0) / 2.0);
        let axis = if angle.abs() < 1e-9 {
            Vector::from_slice(&[0.0, 0.0, 0.0])
        } else {
            let factor = 1.0 / (2.0 * libm::sin(angle));
            Vector::from_slice(&[
                (R_error.m[2][1] - R_error.m[1][2]) * factor,
                (R_error.m[0][2] - R_error.m[2][0]) * factor,
                (R_error.m[1][0] - R_error.m[0][1]) * factor,
            ])
        };

        let mut error = Vector::<6>::zero();
        let orientation_error = axis * angle;
        error[0] = orientation_error[0];
        error[1] = orientation_error[1];
        error[2] = orientation_error[2];
        error[3] = p_error.x;
        error[4] = p_error.y;
        error[5] = p_error.z;

        error
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinematics::math::se3::Twist;
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
    fn test_dls_ik_simple_target() {
        let kinematics = get_test_kinematics();
        let target_angles = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6];
        let target_transform = kinematics.forward(&target_angles).unwrap();

        let initial_angles = [0.0; 6];
        let solved_angles = kinematics
            .inverse_dls(&target_transform, &initial_angles, 0.1)
            .unwrap();

        let solved_transform = kinematics.forward(&solved_angles).unwrap();
        let error = PoeKinematics::<6>::calculate_error(&target_transform, &solved_transform);

        assert!(error.norm() < 1e-3);
    }
}
