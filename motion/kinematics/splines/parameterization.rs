//! Arc-length parameterization and sampling for splines.
#![no_std]

use crate::motion::kinematics::splines::math::{de_boor, ControlPoint, SplineError};
use heapless::Vec;
use micromath::F32Ext;

/// The number of steps used to build the arc-length lookup table.
/// A higher number increases accuracy but also computation time and memory usage.
const INTEGRATION_STEPS: usize = 200;

/// The maximum degree for a spline. This is constrained by `de_boor`'s internal array sizes.
const MAX_DEGREE: usize = 10;

/// The maximum number of control points for a spline.
const MAX_CONTROL_POINTS: usize = 50;

/// The maximum number of knots in the knot vector.
const MAX_KNOTS: usize = MAX_CONTROL_POINTS + MAX_DEGREE + 1;

/// Represents a B-Spline or NURBS curve, ready for evaluation and sampling.
pub struct Spline {
    degree: usize,
    knots: Vec<f32, MAX_KNOTS>,
    control_points: Vec<ControlPoint, MAX_CONTROL_POINTS>,
    arc_length_table: Vec<(f32, f32), INTEGRATION_STEPS>, // (u, s)
    total_arc_length: f32,
}

impl Spline {
    /// Creates a new spline and builds its arc-length parameterization table.
    ///
    /// # Arguments
    /// * `degree`: The degree of the spline.
    /// * `knots`: The knot vector.
    /// * `control_points`: The control points.
    ///
    /// # Returns
    /// A `Result` containing the new `Spline` or a `SplineError`.
    pub fn new(
        degree: usize,
        knots: &[f32],
        control_points: &[ControlPoint],
    ) -> Result<Self, SplineError> {
        let mut spline = Self {
            degree,
            knots: Vec::from_slice(knots).unwrap(), // Should not fail with proper constants
            control_points: Vec::from_slice(control_points).unwrap(),
            arc_length_table: Vec::new(),
            total_arc_length: 0.0,
        };
        spline.build_arc_length_table()?;
        Ok(spline)
    }

    /// The domain of the spline parameter `u`.
    fn domain(&self) -> (f32, f32) {
        (
            self.knots[self.degree],
            self.knots[self.control_points.len()],
        )
    }

    /// Evaluates the spline at a given parameter `u`.
    pub fn point(&self, u: f32) -> Result<ControlPoint, SplineError> {
        de_boor(self.degree, &self.knots, &self.control_points, u)
    }

    /// Builds a lookup table mapping the spline parameter `u` to the arc length `s`.
    /// This uses numerical integration (trapezoidal rule) and is essential for constant-speed traversal.
    fn build_arc_length_table(&mut self) -> Result<(), SplineError> {
        self.arc_length_table.clear();
        let (u_start, u_end) = self.domain();
        let step_size = (u_end - u_start) / (INTEGRATION_STEPS as f32 - 1.0);

        let mut current_length = 0.0;
        let mut last_point = self.point(u_start)?;
        self.arc_length_table.push((u_start, 0.0)).unwrap();

        for i in 1..INTEGRATION_STEPS {
            let u = u_start + i as f32 * step_size;
            let current_point = self.point(u)?;

            let dx = current_point.x - last_point.x;
            let dy = current_point.y - last_point.y;
            let dz = current_point.z - last_point.z;
            let segment_length = (dx * dx + dy * dy + dz * dz).sqrt();

            current_length += segment_length;
            self.arc_length_table.push((u, current_length)).unwrap();
            last_point = current_point;
        }

        self.total_arc_length = current_length;
        Ok(())
    }

    /// Converts a desired arc length `s` into the corresponding spline parameter `u`
    /// by linearly interpolating from the arc-length table.
    fn arc_length_to_u(&self, s: f32) -> f32 {
        if s <= 0.0 {
            return self.domain().0;
        }
        if s >= self.total_arc_length {
            return self.domain().1;
        }

        // Find the segment in the lookup table that contains `s`.
        let mut upper_index = 0;
        for i in 1..self.arc_length_table.len() {
            if self.arc_length_table[i].1 >= s {
                upper_index = i;
                break;
            }
        }
        let lower_index = upper_index - 1;

        let (u_lower, s_lower) = self.arc_length_table[lower_index];
        let (u_upper, s_upper) = self.arc_length_table[upper_index];

        // Linearly interpolate to find the estimated `u`.
        let s_fraction = (s - s_lower) / (s_upper - s_lower);
        u_lower + s_fraction * (u_upper - u_lower)
    }

    /// Samples the spline at uniform physical distance intervals.
    ///
    /// # Arguments
    /// * `distance_step`: The physical distance between each sample point.
    /// * `samples`: A mutable `Vec` to store the resulting `ControlPoint` samples.
    ///
    /// # Returns
    /// `Ok(())` on success, or `SplineError` if evaluation fails.
    pub fn sample_by_distance<const N: usize>(
        &self,
        distance_step: f32,
        samples: &mut Vec<ControlPoint, N>,
    ) -> Result<(), SplineError> {
        samples.clear();
        if distance_step <= 0.0 {
            return Ok(());
        }

        let num_steps = (self.total_arc_length / distance_step).floor() as usize;

        // Add the start point
        samples.push(self.point(self.domain().0)?).unwrap();

        for i in 1..=num_steps {
            let s = i as f32 * distance_step;
            let u = self.arc_length_to_u(s);
            samples.push(self.point(u)?).unwrap();
        }

        // Always include the exact end point to avoid floating point errors leaving a gap.
        // Check if the last calculated point is already very close to the end.
        let last_sampled_s = num_steps as f32 * distance_step;
        if (self.total_arc_length - last_sampled_s).abs() > 1e-4 {
             samples.push(self.point(self.domain().1)?).unwrap();
        }

        Ok(())
    }

    /// Returns the total calculated arc length of the spline.
    pub fn total_arc_length(&self) -> f32 {
        self.total_arc_length
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_spline() -> Spline {
        let degree = 3;
        let knots: &[f32] = &[0.0, 0.0, 0.0, 0.0, 1.0, 2.0, 3.0, 3.0, 3.0, 3.0];
        let control_points: &[ControlPoint] = &[
            ControlPoint::new(0.0, 0.0, 0.0),
            ControlPoint::new(10.0, 20.0, 0.0),
            ControlPoint::new(30.0, -10.0, 0.0),
            ControlPoint::new(50.0, 10.0, 0.0),
            ControlPoint::new(60.0, 30.0, 0.0),
            ControlPoint::new(80.0, 0.0, 0.0),
        ];
        Spline::new(degree, knots, control_points).unwrap()
    }

    #[test]
    fn test_spline_creation_and_arc_length() {
        let spline = get_test_spline();
        // The exact length is complex to calculate analytically, but it must be positive.
        // A straight line between start and end would be 80.0. The curve is longer.
        assert!(spline.total_arc_length() > 80.0);
        assert!(!spline.arc_length_table.is_empty());
        assert_eq!(spline.arc_length_table[0], (0.0, 0.0));
        assert!((spline.arc_length_table.last().unwrap().1 - spline.total_arc_length()).abs() < 1e-4);
    }

    #[test]
    fn test_arc_length_to_u_conversion() {
        let spline = get_test_spline();
        let (u_start, u_end) = spline.domain();

        // s=0 should map to u_start
        assert!((spline.arc_length_to_u(0.0) - u_start).abs() < 1e-6);

        // s=total_length should map to u_end
        assert!((spline.arc_length_to_u(spline.total_arc_length()) - u_end).abs() < 1e-4);

        // A point halfway through the arc length should map to a u value between start and end.
        let mid_u = spline.arc_length_to_u(spline.total_arc_length() / 2.0);
        assert!(mid_u > u_start && mid_u < u_end);
    }

    #[test]
    fn test_sample_by_distance() {
        let spline = get_test_spline();
        let distance_step = 10.0;
        let mut samples: Vec<ControlPoint, 20> = Vec::new();

        spline.sample_by_distance(distance_step, &mut samples).unwrap();

        // Verify that the number of points is roughly correct
        let expected_points = (spline.total_arc_length() / distance_step).floor() as usize + 1;
        // +1 potentially for the explicit end point
        assert!(samples.len() >= expected_points && samples.len() <= expected_points + 1);

        // Verify that the distance between consecutive points is approximately `distance_step`
        for i in 0..(samples.len() - 1) {
            let p1 = samples[i];
            let p2 = samples[i+1];
            let dist = ((p2.x - p1.x).powi(2) + (p2.y - p1.y).powi(2)).sqrt();

            // The last segment might be shorter
            if i < samples.len() - 2 {
                 assert!((dist - distance_step).abs() < 0.5, "Distance between points {} and {} is {}, expected {}", i, i+1, dist, distance_step);
            }
        }

        // Verify start and end points
        let start_point = spline.point(spline.domain().0).unwrap();
        let end_point = spline.point(spline.domain().1).unwrap();
        assert!((samples[0].x - start_point.x).abs() < 1e-4);
        assert!((samples.last().unwrap().x - end_point.x).abs() < 1e-4);
    }

    #[test]
    fn test_nurbs_circle_sampling() {
        let degree = 2;
        let knots: &[f32] = &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let w = 1.0 / 2.0f32.sqrt();
        let control_points: &[ControlPoint] = &[
            ControlPoint::new_rational(10.0, 0.0, 0.0, 1.0),
            ControlPoint::new_rational(10.0, 10.0, 0.0, w),
            ControlPoint::new_rational(0.0, 10.0, 0.0, 1.0),
        ];
        let spline = Spline::new(degree, knots, control_points).unwrap();

        // Expected length of a circle quadrant with radius 10 is 2 * pi * 10 / 4 = 5 * pi ~= 15.708
        let expected_length = 5.0 * core::f32::consts::PI;
        assert!((spline.total_arc_length() - expected_length).abs() < 0.01);

        let mut samples: Vec<ControlPoint, 20> = Vec::new();
        spline.sample_by_distance(1.0, &mut samples).unwrap();

        // All sampled points should have a radius of approximately 10.
        for point in samples {
            let radius = (point.x.powi(2) + point.y.powi(2)).sqrt();
            assert!((radius - 10.0).abs() < 0.01, "Point ({}, {}) has radius {}, expected 10.0", point.x, point.y, radius);
        }
    }
}
