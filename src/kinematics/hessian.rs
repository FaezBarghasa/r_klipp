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

use crate::kinematics::jacobian::Jacobian;
use crate::kinematics::poe_fk::PoeKinematics;
use crate::math::{Matrix, Vector, Vector3};

pub type Hessian<const N: usize> = Matrix<{ 6 * N }, N>;

impl<const N: usize> PoeKinematics<N> {
    /// Calculates the time derivative of the Jacobian (dJ/dt).
    /// dJ/dt = sum_{i=1 to N} (Lie_bracket(J_i, J_k) * q_dot_k)
    pub fn jacobian_derivative(
        &self,
        joint_angles: &[f64; N],
        joint_velocities: &[f64; N],
    ) -> Result<Jacobian<N>, &'static str> {
        let mut dJ = Jacobian::<N>::zero();
        let J = self.jacobian(joint_angles)?;

        for i in 0..N {
            let mut dJ_i = Vector::<6>::zero();
            for k in 0..i {
                let J_i_twist = J.get_column(i).to_twist();
                let J_k_twist = J.get_column(k).to_twist();
                let lie_bracket = J_i_twist.lie_bracket(&J_k_twist);
                dJ_i = dJ_i + lie_bracket.to_vector() * joint_velocities[k];
            }
            dJ.set_column(i, &dJ_i);
        }

        Ok(dJ)
    }

    /// Calculates the Kinematic Hessian.
    /// This is a simplified version, more suitable for control than the full tensor.
    pub fn hessian(&self, joint_angles: &[f64; N]) -> Result<Hessian<N>, &'static str> {
        let mut H = Hessian::<N>::zero();
        let J = self.jacobian(joint_angles)?;

        for i in 0..N {
            for j in 0..N {
                let J_i_twist = J.get_column(i).to_twist();
                let J_j_twist = J.get_column(j).to_twist();
                let lie_bracket = J_i_twist.lie_bracket(&J_j_twist);

                for k in 0..6 {
                    H.m[i * 6 + k][j] = lie_bracket[k];
                }
            }
        }
        Ok(H)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinematics::math::se3::{Transform, Twist};
    use crate::math::Matrix4;

    fn get_test_kinematics() -> PoeKinematics<3> {
        let screw_axes = [
            Twist { angular: Vector3::new(0.0, 0.0, 1.0), linear: Vector3::new(0.0, 0.0, 0.0) },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -0.5) },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -1.0) },
        ];
        let M = Transform(Matrix4::identity());
        let joint_limits = [(-3.14, 3.14), (-3.14, 3.14), (-3.14, 3.14)];
        PoeKinematics { screw_axes, M, joint_limits }
    }

    #[test]
    fn test_jacobian_derivative_simple_case() {
        let kinematics = get_test_kinematics();
        let joint_angles = [0.0, core::f64::consts::PI / 2.0, 0.0];
        let joint_velocities = [1.0, 0.5, 0.2];

        let dJ = kinematics.jacobian_derivative(&joint_angles, &joint_velocities).unwrap();

        assert_eq!(dJ.rows(), 6);
        assert_eq!(dJ.cols(), 3);
        // A non-zero velocity should result in a non-zero dJ/dt for a non-trivial robot
        assert!(dJ.norm() > 1e-6);
    }

    #[test]
    fn test_hessian() {
        let kinematics = get_test_kinematics();
        let joint_angles = [0.1, 0.2, 0.3];
        let H = kinematics.hessian(&joint_angles).unwrap();
        assert_eq!(H.rows(), 18);
        assert_eq!(H.cols(), 3);
        assert!(H.norm() > 1e-6);
    }
}