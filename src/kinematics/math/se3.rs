
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

use crate::math::{Matrix4, Point, Vector3};
use libm::{cos, sin};

/// A 6x1 twist vector representing an element of se(3).
/// The first 3 elements are the angular velocity, and the last 3 are the linear velocity.
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Twist {
    pub angular: Vector3,
    pub linear: Vector3,
}

impl Twist {
    pub const ZERO: Self = Self {
        angular: Vector3::ZERO,
        linear: Vector3::ZERO,
    };
}

/// Represents a 4x4 homogeneous transformation matrix, an element of SE(3).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform(pub Matrix4);

impl Transform {
    /// Creates a new identity transform.
    pub fn identity() -> Self {
        Self(Matrix4::identity())
    }

    /// Inverts the transformation matrix.
    /// For SE(3), the inverse is [R^T, -R^T * t; 0, 1].
    pub fn inverse(&self) -> Self {
        let mut R = self.0.get_rotation();
        let t = self.0.get_translation();

        R.transpose_mut(); // R is now R^T

        let t_inv = -(R * t);

        let mut inv = Matrix4::identity();
        inv.set_rotation(&R);
        inv.set_translation(&t_inv);

        Self(inv)
    }

    /// Performs the exponential map from se(3) to SE(3).
    /// Converts a twist and a joint angle into a 4x4 transformation matrix.
    pub fn exp(twist: &Twist, theta: f64) -> Result<Self, &'static str> {
        let w = twist.angular;
        let v = twist.linear;
        let w_norm = w.norm();

        let R = if w_norm < 1e-9 {
            // Pure translation
            Matrix4::identity().get_rotation()
        } else {
            // Use Rodrigues' formula for rotation
            let wx = w.to_skew_symmetric();
            Matrix4::identity().get_rotation()
                + wx * sin(theta * w_norm) / w_norm
                + (wx * wx) * (1.0 - cos(theta * w_norm)) / (w_norm * w_norm)
        };

        let t = if w_norm < 1e-9 {
            // Pure translation
            v * theta
        } else {
            // Full screw motion
            let wx = w.to_skew_symmetric();
            (Matrix4::identity().get_rotation() * theta
                + (1.0 - cos(theta * w_norm)) / (w_norm * w_norm) * wx
                + (theta * w_norm - sin(theta * w_norm)) / (w_norm * w_norm * w_norm) * (wx * wx))
                * v
        };

        let mut transform = Matrix4::identity();
        transform.set_rotation(&R);
        transform.set_translation(&t.into());

        Ok(Self(transform))
    }
}

impl core::ops::Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0 * rhs.0)
    }
}

impl core::ops::Mul<Point> for Transform {
    type Output = Point;

    fn mul(self, rhs: Point) -> Self::Output {
        self.0 * rhs
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::math::{Matrix3, Vector3};

    #[test]
    fn test_transform_identity() {
        let t = Transform::identity();
        assert_eq!(t.0, Matrix4::identity());
    }

    #[test]
    fn test_transform_inverse() {
        let R = Matrix3::from_rows(
            [0.0, -1.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
        );
        let t = Vector3::new(1.0, 2.0, 3.0);
        let mut mat = Matrix4::identity();
        mat.set_rotation(&R);
        mat.set_translation(&t);

        let transform = Transform(mat);
        let inv_transform = transform.inverse();
        let product = transform * inv_transform;

        for i in 0..4 {
            for j in 0..4 {
                assert!((product.0.m[i][j] - Matrix4::identity().m[i][j]).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn test_exp_map_pure_rotation() {
        let twist = Twist {
            angular: Vector3::new(0.0, 0.0, 1.0),
            linear: Vector3::new(0.0, 0.0, 0.0),
        };
        let theta = core::f64::consts::PI / 2.0; // 90 degrees

        let transform = Transform::exp(&twist, theta).unwrap();

        let expected_R = Matrix3::from_rows(
            [cos(theta), -sin(theta), 0.0],
            [sin(theta), cos(theta), 0.0],
            [0.0, 0.0, 1.0],
        );
        let mut expected_mat = Matrix4::identity();
        expected_mat.set_rotation(&expected_R);

        for i in 0..4 {
            for j in 0..4 {
                assert!((transform.0.m[i][j] - expected_mat.m[i][j]).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn test_exp_map_pure_translation() {
        let twist = Twist {
            angular: Vector3::ZERO,
            linear: Vector3::new(1.0, 2.0, 3.0),
        };
        let theta = 2.0;

        let transform = Transform::exp(&twist, theta).unwrap();

        let expected_t = Vector3::new(2.0, 4.0, 6.0);
        let mut expected_mat = Matrix4::identity();
        expected_mat.set_translation(&expected_t);

        assert_eq!(transform.0, expected_mat);
    }
}
