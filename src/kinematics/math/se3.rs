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

use crate::math::{
    matrix_utils::MatrixBlock, Matrix, Matrix3, Matrix4, Point, Vector, Vector3,
};
use libm::{acos, cos, sin, sqrt};

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

    pub fn to_vector(&self) -> Vector<6> {
        Vector::from_slice(&[
            self.angular.x,
            self.angular.y,
            self.angular.z,
            self.linear.x,
            self.linear.y,
            self.linear.z,
        ])
    }

    /// Computes the Lie bracket [V1, V2] of two twists.
    pub fn lie_bracket(&self, other: &Self) -> Self {
        let angular = self.angular.cross(other.angular);
        let linear = self.angular.cross(other.linear) + self.linear.cross(other.angular);
        Self { angular, linear }
    }
}

impl core::ops::Index<usize> for Twist {
    type Output = f64;

    fn index(&self, index: usize) -> &Self::Output {
        match index {
            0 => &self.angular.x,
            1 => &self.angular.y,
            2 => &self.angular.z,
            3 => &self.linear.x,
            4 => &self.linear.y,
            5 => &self.linear.z,
            _ => panic!("Index out of bounds for Twist"),
        }
    }
}

impl From<Vector<6>> for Twist {
    fn from(v: Vector<6>) -> Self {
        Self {
            angular: Vector3::new(v[0], v[1], v[2]),
            linear: Vector3::new(v[3], v[4], v[5]),
        }
    }
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

    /// Computes the Adjoint representation of the transform.
    pub fn adjoint(&self) -> Matrix<6, 6> {
        let R = self.0.get_rotation();
        let t = self.0.get_translation();
        let p_skew = t.to_skew_symmetric();

        let mut Ad_T = Matrix::<6, 6>::zero();
        Ad_T.set_block(0, 0, &R);
        Ad_T.set_block(3, 3, &R);
        Ad_T.set_block(3, 0, &(p_skew * R));

        Ad_T
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

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Quaternion {
    pub w: f64,
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Quaternion {
    pub fn from_matrix(mat: &Matrix3) -> Self {
        let tr = mat.trace();
        let w = sqrt((1.0 + tr).max(0.0)) / 2.0;
        let x = sqrt((1.0 + mat.m[0][0] - mat.m[1][1] - mat.m[2][2]).max(0.0)) / 2.0;
        let y = sqrt((1.0 - mat.m[0][0] + mat.m[1][1] - mat.m[2][2]).max(0.0)) / 2.0;
        let z = sqrt((1.0 - mat.m[0][0] - mat.m[1][1] + mat.m[2][2]).max(0.0)) / 2.0;
        Self {
            w,
            x: x.copysign(mat.m[2][1] - mat.m[1][2]),
            y: y.copysign(mat.m[0][2] - mat.m[2][0]),
            z: z.copysign(mat.m[1][0] - mat.m[0][1]),
        }
    }

    pub fn conjugate(&self) -> Self {
        Self {
            w: self.w,
            x: -self.x,
            y: -self.y,
            z: -self.z,
        }
    }

    pub fn slerp(self, other: Self, t: f64) -> Self {
        let mut cos_theta = self.w * other.w + self.x * other.x + self.y * other.y + self.z * other.z;
        let mut other = other;

        if cos_theta < 0.0 {
            other.w = -other.w;
            other.x = -other.x;
            other.y = -other.y;
            other.z = -other.z;
            cos_theta = -cos_theta;
        }

        if cos_theta > 1.0 - 1e-9 {
            // Linear interpolation for small angles
            return Self {
                w: self.w + t * (other.w - self.w),
                x: self.x + t * (other.x - self.x),
                y: self.y + t * (other.y - self.y),
                z: self.z + t * (other.z - self.z),
            };
        }

        let theta = acos(cos_theta);
        let sin_theta = sin(theta);

        let w1 = sin((1.0 - t) * theta) / sin_theta;
        let w2 = sin(t * theta) / sin_theta;

        Self {
            w: self.w * w1 + other.w * w2,
            x: self.x * w1 + other.x * w2,
            y: self.y * w1 + other.y * w2,
            z: self.z * w1 + other.z * w2,
        }
    }
}

impl core::ops::Mul for Quaternion {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        Self {
            w: self.w * rhs.w - self.x * rhs.x - self.y * rhs.y - self.z * rhs.z,
            x: self.w * rhs.x + self.x * rhs.w + self.y * rhs.z - self.z * rhs.y,
            y: self.w * rhs.y - self.x * rhs.z + self.y * rhs.w + self.z * rhs.x,
            z: self.w * rhs.z + self.x * rhs.y - self.y * rhs.x + self.z * rhs.w,
        }
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

    #[test]
    fn test_quaternion_from_matrix() {
        let R = Matrix3::from_rows(
            [0.0, -1.0, 0.0],
            [1.0, 0.0, 0.0],
            [0.0, 0.0, 1.0],
        );
        let q = Quaternion::from_matrix(&R);
        assert!((q.w - 1.0/sqrt(2.0)).abs() < 1e-9);
        assert!((q.x - 0.0).abs() < 1e-9);
        assert!((q.y - 0.0).abs() < 1e-9);
        assert!((q.z - 1.0/sqrt(2.0)).abs() < 1e-9);
    }

    #[test]
    fn test_slerp() {
        let q1 = Quaternion { w: 1.0, x: 0.0, y: 0.0, z: 0.0 };
        let q2 = Quaternion { w: 0.0, x: 1.0, y: 0.0, z: 0.0 };
        let q_mid = q1.slerp(q2, 0.5);
        assert!((q_mid.w - 1.0/sqrt(2.0)).abs() < 1e-9);
        assert!((q_mid.x - 1.0/sqrt(2.0)).abs() < 1e-9);
    }

    #[test]
    fn test_lie_bracket() {
        let t1 = Twist { angular: Vector3::new(1.0, 0.0, 0.0), linear: Vector3::new(0.0, 1.0, 0.0) };
        let t2 = Twist { angular: Vector3::new(0.0, 1.0, 0.0), linear: Vector3::new(1.0, 0.0, 0.0) };
        let bracket = t1.lie_bracket(&t2);
        assert_eq!(bracket.angular, Vector3::new(0.0, 0.0, 1.0));
        assert_eq!(bracket.linear, Vector3::new(0.0, 0.0, -2.0));
    }

    #[test]
    fn test_adjoint() {
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

        let Ad_T = transform.adjoint();
        assert_eq!(Ad_T.rows(), 6);
        assert_eq!(Ad_T.cols(), 6);
    }
}