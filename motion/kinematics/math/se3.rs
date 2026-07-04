//! Core data structures and mathematical operations for SE(3) Lie Group.
//!
//! This module provides the fundamental building blocks for working with rigid body
//! transformations using 4x4 homogeneous matrices, as well as their Lie algebra
//! counterparts (6x1 twist vectors). All implementations are `#[no_std]` and
//! optimized for real-time embedded environments.

use libm::{cosf, sinf, fabsf};

// A 4x4 homogeneous transformation matrix representing an element of SE(3).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Transform {
    pub rot: [[f32; 3]; 3],
    pub trans: [f32; 3],
}

impl Transform {
    /// Creates a new identity transform.
    pub fn identity() -> Self {
        Self {
            rot: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            trans: [0.0, 0.0, 0.0],
        }
    }

    /// Inverts the transformation.
    /// For SE(3), the inverse is R^T and -R^T * t.
    pub fn inverse(&self) -> Self {
        let mut rot_inv = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                rot_inv[i][j] = self.rot[j][i];
            }
        }

        let mut trans_inv = [0.0; 3];
        trans_inv[0] = - (rot_inv[0][0] * self.trans[0] + rot_inv[0][1] * self.trans[1] + rot_inv[0][2] * self.trans[2]);
        trans_inv[1] = - (rot_inv[1][0] * self.trans[0] + rot_inv[1][1] * self.trans[1] + rot_inv[1][2] * self.trans[2]);
        trans_inv[2] = - (rot_inv[2][0] * self.trans[0] + rot_inv[2][1] * self.trans[1] + rot_inv[2][2] * self.trans[2]);

        Self {
            rot: rot_inv,
            trans: trans_inv,
        }
    }
}

impl core::ops::Mul for Transform {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self {
        let mut rot_res = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                rot_res[i][j] = self.rot[i][0] * rhs.rot[0][j] + self.rot[i][1] * rhs.rot[1][j] + self.rot[i][2] * rhs.rot[2][j];
            }
        }

        let mut trans_res = [0.0; 3];
        trans_res[0] = self.rot[0][0] * rhs.trans[0] + self.rot[0][1] * rhs.trans[1] + self.rot[0][2] * rhs.trans[2] + self.trans[0];
        trans_res[1] = self.rot[1][0] * rhs.trans[0] + self.rot[1][1] * rhs.trans[1] + self.rot[1][2] * rhs.trans[2] + self.trans[1];
        trans_res[2] = self.rot[2][0] * rhs.trans[0] + self.rot[2][1] * rhs.trans[1] + self.rot[2][2] * rhs.trans[2] + self.trans[2];

        Self {
            rot: rot_res,
            trans: trans_res,
        }
    }
}

/// A 6x1 twist vector representing an element of se(3).
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Twist {
    pub v: [f32; 3],
    pub w: [f32; 3],
}

/// Computes the matrix exponential for a twist.
pub fn exponential_map(twist: &Twist, theta: f32) -> Transform {
    let w_norm = fabsf(twist.w[0]*twist.w[0] + twist.w[1]*twist.w[1] + twist.w[2]*twist.w[2]).sqrt();
    if w_norm < 1e-6 { // Pure translation
        return Transform {
            rot: [
                [1.0, 0.0, 0.0],
                [0.0, 1.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            trans: [
                twist.v[0] * theta,
                twist.v[1] * theta,
                twist.v[2] * theta,
            ],
        };
    }

    let w_skew = [
        [0.0, -twist.w[2], twist.w[1]],
        [twist.w[2], 0.0, -twist.w[0]],
        [-twist.w[1], twist.w[0], 0.0],
    ];

    let w_skew_sq = [
        [ -twist.w[1]*twist.w[1] - twist.w[2]*twist.w[2], twist.w[0]*twist.w[1], twist.w[0]*twist.w[2] ],
        [ twist.w[0]*twist.w[1], -twist.w[0]*twist.w[0] - twist.w[2]*twist.w[2], twist.w[1]*twist.w[2] ],
        [ twist.w[0]*twist.w[2], twist.w[1]*twist.w[2], -twist.w[0]*twist.w[0] - twist.w[1]*twist.w[1] ],
    ];

    let s_theta = sinf(theta);
    let c_theta = cosf(theta);

    let mut rot = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            rot[i][j] = if i==j {1.0} else {0.0} + s_theta * w_skew[i][j] + (1.0 - c_theta) * w_skew_sq[i][j];
        }
    }

    let mut g_v = [0.0; 3];
    let g1 = (1.0 - c_theta) / w_norm;
    let g2 = (theta - s_theta) / (w_norm * w_norm);

    let cross_v_w = [
        twist.v[1] * twist.w[2] - twist.v[2] * twist.w[1],
        twist.v[2] * twist.w[0] - twist.v[0] * twist.w[2],
        twist.v[0] * twist.w[1] - twist.v[1] * twist.w[0],
    ];

    let dot_v_w = twist.v[0]*twist.w[0] + twist.v[1]*twist.w[1] + twist.v[2]*twist.w[2];

    for i in 0..3 {
        g_v[i] = theta * twist.v[i] + g1 * cross_v_w[i] + g2 * dot_v_w * twist.w[i];
    }

    Transform {
        rot,
        trans: g_v,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transform_identity() {
        let t = Transform::identity();
        assert_eq!(t.rot, [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]]);
        assert_eq!(t.trans, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn test_transform_inverse() {
        let t = Transform {
            rot: [
                [0.0, -1.0, 0.0],
                [1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
            ],
            trans: [1.0, 2.0, 3.0],
        };
        let t_inv = t.inverse();
        let res = t * t_inv;
        let identity = Transform::identity();

        for i in 0..3 {
            for j in 0..3 {
                assert!((res.rot[i][j] - identity.rot[i][j]).abs() < 1e-6);
            }
            assert!((res.trans[i] - identity.trans[i]).abs() < 1e-6);
        }
    }
}
