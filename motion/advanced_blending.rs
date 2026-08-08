//! C3 continuous corner blending using splines.
#![no_std]

use crate::motion::kinematics::splines::math::ControlPoint;
use heapless::Vec;

/// Defines the kinematic state at a boundary point (start or end of a segment).
/// This is used to ensure smooth transitions.
#[derive(Debug, Clone, Copy)]
pub struct BoundaryCondition {
    /// P: Position (Point)
    pub p: [f32; 3],
    /// P': First derivative (Velocity vector)
    pub p1: [f32; 3],
    /// P'': Second derivative (Acceleration vector)
    pub p2: [f32; 3],
    /// P''': Third derivative (Jerk vector)
    pub p3: [f32; 3],
}

/// Generates a C3 continuous (jerk-continuous) blending spline between two segments.
///
/// This function constructs a 7th-degree Bézier curve that smoothly connects
/// two points (p0 and p1) given their kinematic properties (position, velocity,
/// acceleration, and jerk) at the boundaries. A 7th-degree (8 control points)
/// curve is required to satisfy the four constraints (P, P', P'', P''') at both
/// the start (t=0) and end (t=1) of the curve.
///
/// # Arguments
/// * `p0`: The boundary condition at the start of the blend (end of the first segment).
/// * `p1`: The boundary condition at the end of the blend (start of the second segment).
///
/// # Returns
/// A `Vec` of 8 `ControlPoint`s that define the blending Bézier curve.
///
/// # Mathematical Derivation
/// The control points `B_i` of a Bézier curve of degree `n` can be determined
/// from the derivatives at the endpoints. For a Bézier curve `C(t)`:
/// C(0) = B_0
/// C'(0) = n * (B_1 - B_0)
/// C''(0) = n * (n-1) * (B_2 - 2*B_1 + B_0)
/// C'''(0) = n * (n-1) * (n-2) * (B_3 - 3*B_2 + 3*B_1 - B_0)
///
/// And similarly at t=1:
/// C(1) = B_n
/// C'(1) = n * (B_n - B_{n-1})
/// C''(1) = n * (n-1) * (B_n - 2*B_{n-1} + B_{n-2})
/// C'''(1) = n * (n-1) * (n-2) * (B_n - 3*B_{n-1} + 3*B_{n-2} - B_{n-3})
///
/// We can solve these equations for the control points B_0, B_1, B_2, B_3 and B_7, B_6, B_5, B_4.
/// For n=7:
/// B_0 = P0
/// B_1 = P0 + P0' / 7
/// B_2 = P0'' / 42 + 2*B_1 - B_0
/// B_3 = P0''' / 210 + 3*B_2 - 3*B_1 + B_0
///
/// B_7 = P1
/// B_6 = P1 - P1' / 7
/// B_5 = P1'' / 42 + 2*B_6 - B_7
/// B_4 = P1''' / 210 + 3*B_5 - 3*B_6 + B_7
///
pub fn generate_c3_blend(
    p0: &BoundaryCondition,
    p1: &BoundaryCondition,
) -> Vec<ControlPoint, 8> {
    let n = 7.0;
    let mut b: Vec<ControlPoint, 8> = Vec::new();

    let b0_x = p0.p[0];
    let b0_y = p0.p[1];
    let b0_z = p0.p[2];
    b.push(ControlPoint::new(b0_x, b0_y, b0_z)).unwrap();

    let b1_x = b0_x + p0.p1[0] / n;
    let b1_y = b0_y + p0.p1[1] / n;
    let b1_z = b0_z + p0.p1[2] / n;
    b.push(ControlPoint::new(b1_x, b1_y, b1_z)).unwrap();

    let b2_x = p0.p2[0] / (n * (n - 1.0)) + 2.0 * b1_x - b0_x;
    let b2_y = p0.p2[1] / (n * (n - 1.0)) + 2.0 * b1_y - b0_y;
    let b2_z = p0.p2[2] / (n * (n - 1.0)) + 2.0 * b1_z - b0_z;
    b.push(ControlPoint::new(b2_x, b2_y, b2_z)).unwrap();

    let b3_x = p0.p3[0] / (n * (n - 1.0) * (n - 2.0)) + 3.0 * b2_x - 3.0 * b1_x + b0_x;
    let b3_y = p0.p3[1] / (n * (n - 1.0) * (n - 2.0)) + 3.0 * b2_y - 3.0 * b1_y + b0_y;
    let b3_z = p0.p3[2] / (n * (n - 1.0) * (n - 2.0)) + 3.0 * b2_z - 3.0 * b1_z + b0_z;
    b.push(ControlPoint::new(b3_x, b3_y, b3_z)).unwrap();

    // The end control points are calculated in reverse
    let b7_x = p1.p[0];
    let b7_y = p1.p[1];
    let b7_z = p1.p[2];

    let b6_x = b7_x - p1.p1[0] / n;
    let b6_y = b7_y - p1.p1[1] / n;
    let b6_z = b7_z - p1.p1[2] / n;

    let b5_x = p1.p2[0] / (n * (n - 1.0)) + 2.0 * b6_x - b7_x;
    let b5_y = p1.p2[1] / (n * (n - 1.0)) + 2.0 * b6_y - b7_y;
    let b5_z = p1.p2[2] / (n * (n - 1.0)) + 2.0 * b6_z - b7_z;

    let b4_x = p1.p3[0] / (n * (n - 1.0) * (n - 2.0)) + 3.0 * b5_x - 3.0 * b6_x + b7_x;
    let b4_y = p1.p3[1] / (n * (n - 1.0) * (n - 2.0)) + 3.0 * b5_y - 3.0 * b6_y + b7_y;
    let b4_z = p1.p3[2] / (n * (n - 1.0) * (n - 2.0)) + 3.0 * b5_z - 3.0 * b6_z + b7_z;

    b.push(ControlPoint::new(b4_x, b4_y, b4_z)).unwrap();
    b.push(ControlPoint::new(b5_x, b5_y, b5_z)).unwrap();
    b.push(ControlPoint::new(b6_x, b6_y, b6_z)).unwrap();
    b.push(ControlPoint::new(b7_x, b7_y, b7_z)).unwrap();

    b
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion::kinematics::splines::parameterization::Spline;

    /// Helper to approximate a derivative of a spline at a given `u`.
    fn approximate_derivative(spline: &Spline, u: f32, order: usize) -> [f32; 3] {
        let h = 1e-4;
        if order == 1 {
            let p_plus = spline.point(u + h).unwrap();
            let p_minus = spline.point(u - h).unwrap();
            let x = (p_plus.x - p_minus.x) / (2.0 * h);
            let y = (p_plus.y - p_minus.y) / (2.0 * h);
            let z = (p_plus.z - p_minus.z) / (2.0 * h);
            [x, y, z]
        } else {
            let d_plus = approximate_derivative(spline, u + h, order - 1);
            let d_minus = approximate_derivative(spline, u - h, order - 1);
            let x = (d_plus[0] - d_minus[0]) / (2.0 * h);
            let y = (d_plus[1] - d_minus[1]) / (2.0 * h);
            let z = (d_plus[2] - d_minus[2]) / (2.0 * h);
            [x, y, z]
        }
    }

    #[test]
    fn test_c3_blend_continuity() {
        let p0 = BoundaryCondition {
            p: [0.0, 0.0, 0.0],
            p1: [10.0, 5.0, 0.0],
            p2: [2.0, -3.0, 0.0],
            p3: [0.0, 1.0, 0.0],
        };
        let p1 = BoundaryCondition {
            p: [10.0, 10.0, 0.0],
            p1: [5.0, -5.0, 0.0],
            p2: [-1.0, 0.0, 0.0],
            p3: [2.0, -2.0, 0.0],
        };

        let blend_cps = generate_c3_blend(&p0, &p1);
        assert_eq!(blend_cps.len(), 8);

        // A 7th degree Bézier curve has a uniform knot vector.
        let degree = 7;
        let knots: &[f32] = &[0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0, 1.0];
        let blend_spline = Spline::new(degree, knots, &blend_cps).unwrap();

        // Verify C0: Position
        let start_pos = blend_spline.point(0.0).unwrap();
        let end_pos = blend_spline.point(1.0).unwrap();
        assert!((start_pos.x - p0.p[0]).abs() < 1e-4);
        assert!((start_pos.y - p0.p[1]).abs() < 1e-4);
        assert!((end_pos.x - p1.p[0]).abs() < 1e-4);
        assert!((end_pos.y - p1.p[1]).abs() < 1e-4);

        // Verify C1: Velocity
        let start_vel = approximate_derivative(&blend_spline, 0.0, 1);
        let end_vel = approximate_derivative(&blend_spline, 1.0, 1);
        assert!((start_vel[0] - p0.p1[0]).abs() < 0.1);
        assert!((start_vel[1] - p0.p1[1]).abs() < 0.1);
        assert!((end_vel[0] - p1.p1[0]).abs() < 0.1);
        assert!((end_vel[1] - p1.p1[1]).abs() < 0.1);

        // Verify C2: Acceleration
        let start_accel = approximate_derivative(&blend_spline, 0.0, 2);
        let end_accel = approximate_derivative(&blend_spline, 1.0, 2);
        assert!((start_accel[0] - p0.p2[0]).abs() < 0.1);
        assert!((start_accel[1] - p0.p2[1]).abs() < 0.1);
        assert!((end_accel[0] - p1.p2[0]).abs() < 0.1);
        assert!((end_accel[1] - p1.p2[1]).abs() < 0.1);

        // Verify C3: Jerk
        let start_jerk = approximate_derivative(&blend_spline, 0.0, 3);
        let end_jerk = approximate_derivative(&blend_spline, 1.0, 3);
        assert!((start_jerk[0] - p0.p3[0]).abs() < 0.1);
        assert!((start_jerk[1] - p0.p3[1]).abs() < 0.1);
        assert!((end_jerk[0] - p1.p3[0]).abs() < 0.1);
        assert!((end_jerk[1] - p1.p3[1]).abs() < 0.1);
    }
}
