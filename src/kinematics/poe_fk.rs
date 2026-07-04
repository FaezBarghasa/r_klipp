
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
use crate::math::Point;

/// Holds the kinematic parameters for a robot defined by the Product of Exponentials formula.
pub struct PoeKinematics<const N: usize> {
    /// Screw axes for each joint in the zero position.
    pub screw_axes: [Twist; N],
    /// Home configuration (end-effector transform when all joint angles are zero).
    pub M: Transform,
}

impl<const N: usize> PoeKinematics<N> {
    /// Calculates the forward kinematics using the Product of Exponentials formula.
    /// T(theta) = exp([S1]*theta1) * ... * exp([Sn]*thetan) * M
    pub fn forward(&self, joint_angles: &[f64; N]) -> Result<Transform, &'static str> {
        let mut transform = Transform::identity();
        for i in 0..N {
            let exp_transform = Transform::exp(&self.screw_axes[i], joint_angles[i])?;
            transform = transform * exp_transform;
        }
        Ok(transform * self.M)
    }

    /// Calculates the forward kinematics incrementally.
    /// This is more efficient if only the last few joints have moved.
    pub fn forward_incremental(
        &self,
        joint_angles: &[f64; N],
        prev_transforms: Option<&[Transform; N]>,
        start_joint: usize,
    ) -> Result<(Transform, [Transform; N]), &'static str> {
        let mut transforms = [Transform::identity(); N];
        let mut current_transform = if start_joint > 0 && prev_transforms.is_some() {
            prev_transforms.unwrap()[start_joint - 1]
        } else {
            Transform::identity()
        };

        for i in start_joint..N {
            let exp_transform = Transform::exp(&self.screw_axes[i], joint_angles[i])?;
            current_transform = current_transform * exp_transform;
            transforms[i] = current_transform;
        }

        let final_transform = current_transform * self.M;
        Ok((final_transform, transforms))
    }

    pub fn get_tcp_position(&self, joint_angles: &[f64; N]) -> Result<Point, &'static str> {
        let transform = self.forward(joint_angles)?;
        Ok(transform * Point::origin())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinematics::math::se3::Twist;
    use crate::math::{Matrix4, Vector3};

    // Example from a 6-axis robot
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
    fn test_poe_fk_zero_angles() {
        let kinematics = get_test_kinematics();
        let joint_angles = [0.0; 6];
        let transform = kinematics.forward(&joint_angles).unwrap();
        assert_eq!(transform, kinematics.M);
    }

    #[test]
    fn test_poe_fk_first_joint_90_deg() {
        let kinematics = get_test_kinematics();
        let mut joint_angles = [0.0; 6];
        joint_angles[0] = core::f64::consts::PI / 2.0;

        let transform = kinematics.forward(&joint_angles).unwrap();

        let exp_s1 = Transform::exp(&kinematics.screw_axes[0], joint_angles[0]).unwrap();
        let expected_transform = exp_s1 * kinematics.M;

        for i in 0..4 {
            for j in 0..4 {
                assert!((transform.0.m[i][j] - expected_transform.0.m[i][j]).abs() < 1e-9);
            }
        }
    }
}
