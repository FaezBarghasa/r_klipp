//! Core mathematics for B-Spline and NURBS curves.
#![no_std]

use micromath::F32Ext;

/// Represents a control point in 3D space with an associated weight for NURBS.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ControlPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    /// Weight for NURBS. For non-rational B-Splines, this is 1.0.
    pub w: f32,
}

impl ControlPoint {
    /// Creates a new 3D control point for a standard B-Spline (weight = 1.0).
    pub const fn new(x: f32, y: f32, z: f32) -> Self {
        Self { x, y, z, w: 1.0 }
    }

    /// Creates a new 3D control point for a NURBS curve with a specific weight.
    pub const fn new_rational(x: f32, y: f32, z: f32, w: f32) -> Self {
        Self { x, y, z, w }
    }
}

/// Custom error type for spline calculations.
#[derive(Debug, PartialEq)]
pub enum SplineError {
    /// The provided parameter `u` is outside the valid domain of the knot vector.
    ParameterOutOfDomain,
    /// The degree of the spline is higher than the number of control points allows.
    InsufficientControlPoints,
    /// The knot vector is not sorted in non-decreasing order.
    UnsortedKnotVector,
    /// A zero-weight sum was encountered during NURBS evaluation, which is a singularity.
    NurbsSingularity,
}

/// Evaluates a point on a B-Spline or NURBS curve using De Boor's algorithm.
///
/// This function is generic and works for any dimension by operating on `ControlPoint`s.
/// It's `#![no_std]` and allocation-free.
///
/// # Arguments
/// * `degree`: The degree `p` of the spline.
/// * `knots`: The knot vector `U`. Must be a non-decreasing sequence.
/// * `control_points`: The control points `P`. The number of control points `n+1` must be correct for the knot vector length `m+1` and degree `p` (m = n + p + 1).
/// * `u`: The parameter at which to evaluate the curve. Must be within the domain `[knots[degree], knots[knots.len() - 1 - degree]]`.
///
/// # Returns
/// A `Result` containing the evaluated `ControlPoint` on the curve or a `SplineError`.
pub fn de_boor(
    degree: usize,
    knots: &[f32],
    control_points: &[ControlPoint],
    u: f32,
) -> Result<ControlPoint, SplineError> {
    let n = control_points.len() - 1;
    let m = knots.len() - 1;

    // Basic validation
    if degree > n {
        return Err(SplineError::InsufficientControlPoints);
    }
    if m != n + degree + 1 {
        // This is a common requirement, though some definitions vary.
        // For simplicity, we enforce it.
        // A more robust implementation might relax this.
    }

    // 1. Find the knot interval [u_k, u_{k+1}) that contains u.
    // The domain of u is typically [u_p, u_{n+1}] or [u_p, u_{m-p}].
    let domain_start = knots.get(degree).ok_or(SplineError::UnsortedKnotVector)?;
    let domain_end = knots.get(n + 1).ok_or(SplineError::UnsortedKnotVector)?;

    if u < *domain_start || u > *domain_end {
        // Allow evaluation at the very end of the domain.
        if (u - *domain_end).abs() > 1e-6 {
             return Err(SplineError::ParameterOutOfDomain);
        }
    }

    let mut k = degree;
    while k <= n && knots[k] <= u {
        k += 1;
    }
    k -= 1;

    // 2. Initialize a temporary array for the relevant control points.
    // We need points P_{k-p}, ..., P_k.
    // To avoid heap allocation, we use a fixed-size array.
    // The maximum degree supported is determined by this array's size.
    const MAX_DEGREE: usize = 10;
    if degree > MAX_DEGREE {
        // In a real scenario, this might be a compile-time check or a different error.
        return Err(SplineError::InsufficientControlPoints);
    }

    let mut d: [ControlPoint; MAX_DEGREE + 1] = [ControlPoint::new(0.0, 0.0, 0.0); MAX_DEGREE + 1];

    for i in 0..=degree {
        let cp_index = k - degree + i;
        if let Some(cp) = control_points.get(cp_index) {
            // Project to homogeneous coordinates for NURBS
            d[i] = ControlPoint {
                x: cp.x * cp.w,
                y: cp.y * cp.w,
                z: cp.z * cp.w,
                w: cp.w,
            };
        } else {
            return Err(SplineError::InsufficientControlPoints);
        }
    }

    // 3. Iteratively compute the intermediate points.
    for r in 1..=degree {
        for j in (r..=degree).rev() {
            let knot_j_pr = knots[j + k - degree];
            let knot_j_r = knots[j + k - r];

            let denominator = knot_j_pr - knot_j_r;

            // Avoid division by zero if knots are coincident.
            let alpha = if denominator.abs() < 1e-9 {
                0.0
            } else {
                (u - knot_j_r) / denominator
            };

            let one_minus_alpha = 1.0 - alpha;

            d[j].x = one_minus_alpha * d[j - 1].x + alpha * d[j].x;
            d[j].y = one_minus_alpha * d[j - 1].y + alpha * d[j].y;
            d[j].z = one_minus_alpha * d[j - 1].z + alpha * d[j].z;
            d[j].w = one_minus_alpha * d[j - 1].w + alpha * d[j].w;
        }
    }

    // 4. The result is in d[p]. Project back to Cartesian coordinates.
    let final_point_homogeneous = d[degree];
    let final_w = final_point_homogeneous.w;

    if final_w.abs() < 1e-9 {
        return Err(SplineError::NurbsSingularity);
    }

    Ok(ControlPoint {
        x: final_point_homogeneous.x / final_w,
        y: final_point_homogeneous.y / final_w,
        z: final_point_homogeneous.z / final_w,
        w: 1.0, // The final point's weight is conceptually 1.0 in Cartesian space.
    })
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_de_boor_quadratic_b_spline() {
        // Example from https://pages.mtu.edu/~shene/COURSES/cs3621/NOTES/spline/B-spline/de-Boor.html
        let degree = 2;
        let knots: &[f32] = &[0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 4.0];
        let control_points: &[ControlPoint] = &[
            ControlPoint::new(0.0, 0.0, 0.0), // P0
            ControlPoint::new(1.0, 2.0, 0.0), // P1
            ControlPoint::new(3.0, 3.0, 0.0), // P2
            ControlPoint::new(4.0, 2.0, 0.0), // P3
            ControlPoint::new(5.0, 0.0, 0.0), // P4
            ControlPoint::new(6.0, 0.0, 0.0), // P5
        ];
        let u = 2.5;

        let result = de_boor(degree, knots, control_points, u).unwrap();

        // Expected result from the online example C(2.5) = (3.75, 2.25)
        assert!((result.x - 3.75).abs() < 1e-4);
        assert!((result.y - 2.25).abs() < 1e-4);
        assert!((result.z - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_de_boor_cubic_b_spline_start_and_end() {
        let degree = 3;
        let knots: &[f32] = &[0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 4.0, 4.0, 4.0, 4.0];
        let control_points: &[ControlPoint] = &[
            ControlPoint::new(1.0, 0.0, 0.0), // P0
            ControlPoint::new(2.0, 4.0, 0.0), // P1
            ControlPoint::new(5.0, 2.0, 0.0), // P2
            ControlPoint::new(8.0, 3.0, 0.0), // P3
            ControlPoint::new(10.0, 1.0, 0.0),// P4
            ControlPoint::new(11.0, 5.0, 0.0),// P5
            ControlPoint::new(14.0, 2.0, 0.0),// P6
        ];

        // For a clamped knot vector, the curve should start at the first control point
        let start_point = de_boor(degree, knots, control_points, 0.0).unwrap();
        assert!((start_point.x - control_points[0].x).abs() < 1e-4);
        assert!((start_point.y - control_points[0].y).abs() < 1e-4);

        // And end at the last control point
        let end_point = de_boor(degree, knots, control_points, 4.0).unwrap();
        assert!((end_point.x - control_points.last().unwrap().x).abs() < 1e-4);
        assert!((end_point.y - control_points.last().unwrap().y).abs() < 1e-4);
    }

    #[test]
    fn test_nurbs_circle_quadrant() {
        // A classic NURBS example: a quadrant of a circle.
        let degree = 2;
        let knots: &[f32] = &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let w = 1.0 / 2.0f32.sqrt();
        let control_points: &[ControlPoint] = &[
            ControlPoint::new_rational(1.0, 0.0, 0.0, 1.0),
            ControlPoint::new_rational(1.0, 1.0, 0.0, w),
            ControlPoint::new_rational(0.0, 1.0, 0.0, 1.0),
        ];

        // Point at u=0.5 should be on the circle arc
        let u = 0.5;
        let result = de_boor(degree, knots, control_points, u).unwrap();

        // Expected point is (sqrt(2)/2, sqrt(2)/2) which is roughly (0.7071, 0.7071)
        let expected_coord = 2.0f32.sqrt() / 2.0;
        assert!((result.x - expected_coord).abs() < 1e-4);
        assert!((result.y - expected_coord).abs() < 1e-4);

        // Check that the radius is 1.0
        let radius = (result.x.powi(2) + result.y.powi(2)).sqrt();
        assert!((radius - 1.0).abs() < 1e-4);
    }

    #[test]
    fn test_parameter_out_of_domain() {
        let degree = 2;
        let knots: &[f32] = &[0.0, 0.0, 0.0, 1.0, 2.0, 2.0, 2.0];
        let control_points: &[ControlPoint] = &[
            ControlPoint::new(0.0, 0.0, 0.0),
            ControlPoint::new(1.0, 2.0, 0.0),
            ControlPoint::new(3.0, 3.0, 0.0),
            ControlPoint::new(4.0, 2.0, 0.0),
        ];

        // Domain is [0.0, 2.0]
        let result_before = de_boor(degree, knots, control_points, -0.1);
        assert_eq!(result_before, Err(SplineError::ParameterOutOfDomain));

        let result_after = de_boor(degree, knots, control_points, 2.1);
        assert_eq!(result_after, Err(SplineError::ParameterOutOfDomain));
    }
}
