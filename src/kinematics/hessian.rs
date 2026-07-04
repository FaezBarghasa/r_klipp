
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
use crate::math::{Matrix, Vector3};

pub type Hessian<const N: usize> = Matrix<{ 6 * N }, N>;

impl<const N: usize> PoeKinematics<N> {
    /// Calculates the time derivative of the Jacobian (dJ/dt).
    pub fn jacobian_derivative(
        &self,
        joint_angles: &[f64; N],
        joint_velocities: &[f64; N],
    ) -> Result<Jacobian<N>, &'static str> {
        let mut dJ = Jacobian::<N>::zero();
        let J = self.jacobian(joint_angles)?;

        for i in 0..N {
            let mut dJ_i = Vector3::ZERO;
            for j in 0..i {
                let J_j_w = Vector3::new(J.m[0][j], J.m[1][j], J.m[2][j]);
                let J_i_v = Vector3::new(J.m[3][i], J.m[4][i], J.m[5][i]);
                dJ_i = dJ_i + J_j_w.cross(J_i_v) * joint_velocities[j];
            }

            let dJ_i_w = Vector3::ZERO; // For revolute joints, the angular part of the Jacobian column is constant
            let dJ_i_v = dJ_i;

            let mut col = dJ.get_column_mut(i);
            col[0] = dJ_i_w.x;
            col[1] = dJ_i_w.y;
            col[2] = dJ_i_w.z;
            col[3] = dJ_i_v.x;
            col[4] = dJ_i_v.y;
            col[5] = dJ_i_v.z;
        }

        Ok(dJ)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinematics::math::se3::Twist;
    use crate::math::Matrix4;

    fn get_test_kinematics() -> PoeKinematics<3> {
        let screw_axes = [
            Twist { angular: Vector3::new(0.0, 0.0, 1.0), linear: Vector3::new(0.0, 0.0, 0.0) },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -0.5) },
            Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(0.0, 0.0, -1.0) },
        ];
        let M = Transform(Matrix4::identity());
        PoeKinematics { screw_axes, M }
    }

    #[test]
    fn test_jacobian_derivative_simple_case() {
        let kinematics = get_test_kinematics();
        let joint_angles = [0.0, core::f64::consts::PI / 2.0, 0.0];
        let joint_velocities = [1.0, 0.5, 0.2];

        let dJ = kinematics.jacobian_derivative(&joint_angles, &joint_velocities).unwrap();

        // A full analytical verification is complex. This test mainly checks for panics
        // and that the output is of the correct shape and seems reasonable.
        assert_eq!(dJ.rows(), 6);
        assert_eq!(dJ.cols(), 3);

        // For a simple 3R manipulator, we can make some assertions.
        // dJ1/dt should be zero.
        let col0 = dJ.get_column(0);
        for i in 0..6 {
            assert_eq!(col0[i], 0.0);
        }
    }
}
