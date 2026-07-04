
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

const SC_IK_MAX_ITERATIONS: usize = 10;
const SC_IK_TOLERANCE: f64 = 1e-4;
const SINGULARITY_THRESHOLD: f64 = 0.01;

impl<const N: usize> PoeKinematics<N> {
    /// Solves the inverse kinematics using a singularity-consistent method.
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
            let singular_values = svd.singular_values();

            let mut J_inv = Matrix::<N, 6>::zero();
            let mut lambda = 0.0;

            for i in 0..singular_values.len() {
                if singular_values[i] < SINGULARITY_THRESHOLD {
                    lambda = 1.0 - (singular_values[i] / SINGULARITY_THRESHOLD).powi(2);
                }
                let s_inv = singular_values[i] / (singular_values[i].powi(2) + lambda.powi(2));
                J_inv = J_inv + svd.V.get_column(i).outer_product(&svd.U.get_column(i)) * s_inv;
            }

            let delta_theta: Vector<N> = J_inv * error;

            for i in 0..N {
                current_angles[i] += delta_theta[i];
            }
        }

        Err("SC-IK failed to converge")
    }

    fn calculate_error_sc(target: &Transform, current: &Transform) -> Vector<6> {
        let error_transform = *target * current.inverse();
        let p_error = error_transform.0.get_translation();

        let q_target = Quaternion::from_matrix(&target.0.get_rotation());
        let q_current = Quaternion::from_matrix(&current.0.get_rotation());

        let q_error = q_target.slerp(q_current, -1.0); // Simplified conjugate
        let orientation_error = Vector::from_slice(&[q_error.x, q_error.y, q_error.z]) * 2.0;

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
