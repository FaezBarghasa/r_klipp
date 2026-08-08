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

use crate::kinematics::math::se3::{Quaternion, Transform};
use crate::kinematics::poe_fk::PoeKinematics;
use crate::math::{Matrix, Vector, SVD};

const SC_IK_MAX_ITERATIONS: usize = 15;
const SC_IK_TOLERANCE: f64 = 1e-5;
const SINGULARITY_THRESHOLD: f64 = 0.02;
const TASK_PRIORITY_DAMPING: f64 = 0.1;

impl<const N: usize> PoeKinematics<N> {
    pub fn inverse_sc(
        &self,
        target_transform: &Transform,
        initial_joint_angles: &[f64; N],
    ) -> Result<[f64; N], &'static str> {
        let mut current_angles = *initial_joint_angles;

        for _ in 0..SC_IK_MAX_ITERATIONS {
            let current_transform = self.forward(&current_angles)?;
            let error = Self::calculate_error_sc(target_transform, &current_transform);

            if error.norm() < SC_IK_TOLERANCE {
                return Ok(current_angles);
            }

            let J = self.jacobian(&current_angles)?;
            let svd = SVD::new(&J);

            let J_inv = svd.pseudo_inverse(SINGULARITY_THRESHOLD);
            let delta_theta_main: Vector<N> = J_inv * error;

            // Null-space projection for secondary task (e.g., joint limit avoidance)
            let I = Matrix::<N, N>::identity();
            let null_space_projector = I - J_inv * J;

            // Example secondary task: move away from joint limits
            let mut grad_H = Vector::<N>::zero();
            for i in 0..N {
                // A simple quadratic cost function for joint limits
                let mid_range = (self.joint_limits[i].0 + self.joint_limits[i].1) / 2.0;
                grad_H[i] = current_angles[i] - mid_range;
            }

            let delta_theta_null: Vector<N> = null_space_projector * (grad_H * -TASK_PRIORITY_DAMPING);

            let delta_theta = delta_theta_main + delta_theta_null;

            for i in 0..N {
                current_angles[i] += delta_theta[i];
                // Clamp to joint limits
                current_angles[i] = current_angles[i].clamp(self.joint_limits[i].0, self.joint_limits[i].1);
            }
        }

        Err("SC-IK failed to converge")
    }

    fn calculate_error_sc(target: &Transform, current: &Transform) -> Vector<6> {
        let p_error = target.0.get_translation() - current.0.get_translation();

        let q_target = Quaternion::from_matrix(&target.0.get_rotation());
        let q_current = Quaternion::from_matrix(&current.0.get_rotation());

        // Shortest path quaternion error
        let q_error = q_target * q_current.conjugate();
        let angle = 2.0 * libm::acos(q_error.w);
        let axis = if angle.abs() < 1e-6 {
            Vector::from_slice(&[0.0, 0.0, 0.0])
        } else {
            Vector::from_slice(&[q_error.x, q_error.y, q_error.z]) / libm::sin(angle / 2.0)
        };
        let orientation_error = axis * angle;

        let mut error = Vector::<6>::zero();
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

    fn get_ur5_kinematics() -> PoeKinematics<6> {
        let screw_axes = [
            Twist { angular: Vector3::new(0.0, 0.0, 1.0), linear: Vector3::new(0.0, 0.0, 0.0) },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -0.089159) },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -0.514159) },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -0.906159) },
            Twist { angular: Vector3::new(0.0, 0.0, -1.0), linear: Vector3::new(0.0, 0.0, 0.0) },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -1.0) },
        ];
        let M = Transform(Matrix4::from_translation(&Vector3::new(0.0, -0.10915, 1.0)));
        let joint_limits = [
            (-3.14, 3.14),
            (-3.14, 3.14),
            (-3.14, 3.14),
            (-3.14, 3.14),
            (-3.14, 3.14),
            (-3.14, 3.14),
        ];
        PoeKinematics { screw_axes, M, joint_limits }
    }

    #[test]
    fn test_sc_ik_convergence() {
        let kinematics = get_ur5_kinematics();
        let target_angles = [0.1, -0.5, 0.2, 0.5, 1.0, 0.3];
        let target_transform = kinematics.forward(&target_angles).unwrap();

        let initial_angles = [0.0; 6];
        let result_angles = kinematics.inverse_sc(&target_transform, &initial_angles).unwrap();

        let result_transform = kinematics.forward(&result_angles).unwrap();
        let error = (target_transform.0 - result_transform.0).norm();
        assert!(error < 1e-4, "IK failed to converge to target");
    }

    #[test]
    fn test_sc_ik_near_singularity() {
        let kinematics = get_ur5_kinematics();
        // Wrist singularity for UR5
        let target_angles = [0.1, -0.5, 0.2, 0.5, 0.0, 0.3];
        let target_transform = kinematics.forward(&target_angles).unwrap();

        let initial_angles = [0.0; 6];
        let result_angles = kinematics.inverse_sc(&target_transform, &initial_angles).unwrap();

        let result_transform = kinematics.forward(&result_angles).unwrap();
        let error = (target_transform.0 - result_transform.0).norm();
        assert!(error < 1e-4, "IK failed to converge near singularity");
    }
}