
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

use crate::kinematics::math::se3::{Transform, Twist};
use crate::kinematics::poe_fk::PoeKinematics;
use crate::math::{Matrix, Point, Vector3};

pub type Jacobian<const N: usize> = Matrix<6, N>;

impl<const N: usize> PoeKinematics<N> {
    /// Calculates the analytical Jacobian for the given joint angles.
    pub fn jacobian(&self, joint_angles: &[f64; N]) -> Result<Jacobian<N>, &'static str> {
        let mut J = Jacobian::<N>::zero();
        let mut T = Transform::identity();

        // Pre-calculate transforms
        let mut transforms = [Transform::identity(); N];
        for i in 0..N {
            let exp_transform = Transform::exp(&self.screw_axes[i], joint_angles[i])?;
            T = T * exp_transform;
            transforms[i] = T;
        }
        let T_final = T * self.M;
        let p_tcp = T_final * Point::origin();

        // Calculate Jacobian columns
        let mut T_i = Transform::identity();
        for i in 0..N {
            let p_i = T_i * Point::origin();
            let s_i = self.screw_axes[i];

            let J_v = s_i.angular.cross(p_tcp - p_i) + s_i.linear;
            let J_w = s_i.angular;

            J.set_column(i, &[J_w.x, J_w.y, J_w.z, J_v.x, J_v.y, J_v.z]);

            T_i = transforms[i];
        }

        Ok(J)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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

        let mut M = Matrix4::identity();
        M.set_translation(&Vector3::new(0.0, -0.109, 1.014));
        let M_transform = Transform(M);

        PoeKinematics {
            screw_axes,
            M: M_transform,
        }
    }

    #[test]
    fn test_jacobian_at_zero_config() {
        let kinematics = get_test_kinematics();
        let joint_angles = [0.0; 6];
        let J = kinematics.jacobian(&joint_angles).unwrap();

        // At the zero configuration, the Jacobian calculation is simpler.
        // This test provides a basic sanity check.
        let p_tcp = kinematics.M * Point::origin();

        for i in 0..6 {
            let s_i = kinematics.screw_axes[i];
            let p_i = Point::origin(); // All joints at origin in zero config for this robot

            let J_v = s_i.angular.cross(p_tcp - p_i) + s_i.linear;
            let J_w = s_i.angular;

            let col = J.get_column(i);
            assert!((col[0] - J_w.x).abs() < 1e-9);
            assert!((col[1] - J_w.y).abs() < 1e-9);
            assert!((col[2] - J_w.z).abs() < 1e-9);
            assert!((col[3] - J_v.x).abs() < 1e-9);
            assert!((col[4] - J_v.y).abs() < 1e-9);
            assert!((col[5] - J_v.z).abs() < 1e-9);
        }
    }
}
