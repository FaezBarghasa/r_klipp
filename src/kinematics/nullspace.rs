
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
use crate::math::{Matrix, Vector};

impl<const N: usize> PoeKinematics<N> {
    /// Calculates the nullspace projector P = I - J^+ * J.
    pub fn nullspace_projector(
        &self,
        joint_angles: &[f64; N],
        lambda: f64,
    ) -> Result<Matrix<N, N>, &'static str> {
        let J = self.jacobian(joint_angles)?;
        let J_t = J.transpose();
        let JJ_t = J * J_t;
        let damping = Matrix::<6, 6>::identity() * (lambda * lambda);
        let inv_term = (JJ_t + damping).inverse().ok_or("Matrix inversion failed")?;
        let J_plus = J_t * inv_term;

        let P = Matrix::<N, N>::identity() - J_plus * J;
        Ok(P)
    }

    /// Calculates the joint velocities for a secondary objective, projected into the nullspace.
    pub fn nullspace_joint_velocities(
        &self,
        joint_angles: &[f64; N],
        lambda: f64,
        secondary_objective: &Vector<N>,
    ) -> Result<Vector<N>, &'static str> {
        let P = self.nullspace_projector(joint_angles, lambda)?;
        let q_dot_null = P * *secondary_objective;
        Ok(q_dot_null)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinematics::math::se3::{Transform, Twist};
    use crate::math::{Matrix4, Vector3};

    // A redundant 7-axis robot
    fn get_redundant_kinematics() -> PoeKinematics<7> {
        let screw_axes = [
            Twist { angular: Vector3::new(0.0, 0.0, 1.0), linear: Vector3::ZERO },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::ZERO },
            Twist { angular: Vector3::new(0.0, 0.0, 1.0), linear: Vector3::ZERO },
            Twist { angular: Vector3::new(0.0, -1.0, 0.0), linear: Vector3::new(0.0, 0.0, -0.5) },
            Twist { angular: Vector3::new(0.0, 0.0, 1.0), linear: Vector3::ZERO },
            Twist { angular: Vector3::new(0.0, -1.0, 0.0), linear: Vector3::new(0.0, 0.0, -1.0) },
            Twist { angular: Vector3::new(0.0, 0.0, 1.0), linear: Vector3::ZERO },
        ];
        let M = Transform(Matrix4::from_translation(&Vector3::new(0.0, 0.0, 1.2)));
        PoeKinematics { screw_axes, M }
    }

    #[test]
    fn test_nullspace_projection() {
        let kinematics = get_redundant_kinematics();
        let joint_angles = [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7];

        // Secondary objective: move joint 3 towards 0.
        let mut secondary_objective = Vector::<7>::zero();
        secondary_objective[3] = -joint_angles[3]; // Gradient descent towards 0

        let q_dot_null = kinematics
            .nullspace_joint_velocities(&joint_angles, 0.01, &secondary_objective)
            .unwrap();

        // The resulting joint velocities should not affect the end-effector pose.
        let J = kinematics.jacobian(&joint_angles).unwrap();
        let tcp_velocity = J * q_dot_null;

        // The TCP velocity should be very close to zero.
        assert!(tcp_velocity.norm() < 1e-6);
    }
}
