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
use crate::math::{Matrix, Vector};

pub type Jacobian<const N: usize> = Matrix<6, N>;

impl<'a, const N: usize> Matrix<6, N> {
    pub fn get_column(&self, j: usize) -> Vector<6> {
        let mut v = Vector::<6>::zero();
        for i in 0..6 {
            v[i] = self.m[i][j];
        }
        v
    }
}

impl Vector<6> {
    pub fn to_twist(&self) -> Twist {
        Twist::from(*self)
    }
}

impl<const N: usize> PoeKinematics<N> {
    /// Calculates the analytical Jacobian for the given joint angles.
    pub fn jacobian(&self, joint_angles: &[f64; N]) -> Result<Jacobian<N>, &'static str> {
        let mut J = Jacobian::<N>::zero();
        let mut T = Transform::identity();

        for i in 0..N {
            let S_i = self.screw_axes[i];
            J.set_column(i, &(T.adjoint() * S_i.to_vector()));
            let exp_transform = Transform::exp(&self.screw_axes[i], joint_angles[i])?;
            T = T * exp_transform;
        }

        Ok(J)
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

        let mut M = Matrix4::identity();
        M.set_translation(&Vector3::new(0.0, -0.109, 1.014));
        let M_transform = Transform(M);
        let joint_limits = [
            (-3.14, 3.14),
            (-3.14, 3.14),
            (-3.14, 3.14),
            (-3.14, 3.14),
            (-3.14, 3.14),
            (-3.14, 3.14),
        ];

        PoeKinematics {
            screw_axes,
            M: M_transform,
            joint_limits,
        }
    }

    #[test]
    fn test_jacobian_at_zero_config() {
        let kinematics = get_test_kinematics();
        let joint_angles = [0.0; 6];
        let J = kinematics.jacobian(&joint_angles).unwrap();

        // At the zero configuration, the Jacobian is just the screw axes.
        for i in 0..6 {
            let s_i = kinematics.screw_axes[i].to_vector();
            let col = J.get_column(i);
            assert!((col - s_i).norm() < 1e-9);
        }
    }
}