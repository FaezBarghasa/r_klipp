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

use crate::math::{Point, Vector3};

const MAX_ITERATIONS_CLOSEST_POINT: usize = 5;
const TOLERANCE_CLOSEST_POINT: f64 = 1e-6;

/// Calculates the Frenet-Serret frame (T, N, B) for a given path.
pub fn frenet_serret_frame(
    path_derivative: Vector3,
    path_second_derivative: Vector3,
) -> (Vector3, Vector3, Vector3) {
    let T = path_derivative.normalize();
    let B_unnormalized = path_derivative.cross(path_second_derivative);
    let B = if B_unnormalized.norm() < 1e-9 {
        // Handle collinear case (straight line) by creating an arbitrary perpendicular vector.
        let mut arbitrary_vec = Vector3::new(1.0, 0.0, 0.0);
        if T.cross(arbitrary_vec).norm() < 1e-9 {
            arbitrary_vec = Vector3::new(0.0, 1.0, 0.0);
        }
        T.cross(arbitrary_vec).normalize()
    } else {
        B_unnormalized.normalize()
    };
    let N = B.cross(T);
    (T, N, B)
}

/// Calculates the contour error by projecting the position error onto the Frenet-Serret frame.
pub fn contour_error(
    position_error: Vector3,
    frenet_frame: &(Vector3, Vector3, Vector3),
) -> f64 {
    let (_T, N, B) = frenet_frame;
    let error_normal = position_error.dot(*N);
    let error_binormal = position_error.dot(*B);
    libm::sqrt(error_normal * error_normal + error_binormal * error_binormal)
}

/// Finds the closest point on a path to a given point using Newton-Raphson.
pub fn find_closest_point_on_path<F, FPrime>(
    path_evaluator: F,
    path_derivative_evaluator: FPrime,
    point: Point,
    initial_u: f64,
) -> f64
where
    F: Fn(f64) -> Point,
    FPrime: Fn(f64) -> Vector3,
{
    let mut u = initial_u;
    for _ in 0..MAX_ITERATIONS_CLOSEST_POINT {
        let p = path_evaluator(u);
        let p_prime = path_derivative_evaluator(u);
        let error_vec = p - point;
        let dot_product = error_vec.dot(p_prime);

        if dot_product.abs() < TOLERANCE_CLOSEST_POINT {
            break;
        }

        let p_double_prime = (path_derivative_evaluator(u + 1e-6) - p_prime) / 1e-6;
        let derivative_of_dot = p_prime.dot(p_prime) + error_vec.dot(p_double_prime);
        u -= dot_product / derivative_of_dot;
    }
    u
}

#[cfg(test)]
mod tests {
    use super::*;

    // A simple circular path for testing
    fn circle_path(u: f64) -> Point {
        Point::new(libm::cos(u), libm::sin(u), 0.0)
    }

    fn circle_path_derivative(u: f64) -> Vector3 {
        Vector3::new(-libm::sin(u), libm::cos(u), 0.0)
    }

    #[test]
    fn test_closest_point_on_circle() {
        let point = Point::new(1.5, 0.5, 0.0);
        let initial_u = 0.0;
        let closest_u = find_closest_point_on_path(circle_path, circle_path_derivative, point, initial_u);

        // The closest point on the circle to (1.5, 0.5) should be on the vector from origin to point
        let expected_u = libm::atan2(0.5, 1.5);
        assert!((closest_u - expected_u).abs() < 1e-5);
    }

    #[test]
    fn test_frenet_frame_circle() {
        let u = core::f64::consts::PI / 4.0;
        let p_prime = circle_path_derivative(u);
        let p_double_prime = Vector3::new(-libm::cos(u), -libm::sin(u), 0.0);
        let (T, N, B) = frenet_serret_frame(p_prime, p_double_prime);

        let sqrt2_inv = 1.0 / libm::sqrt(2.0);
        assert!((T - Vector3::new(-sqrt2_inv, sqrt2_inv, 0.0)).norm() < 1e-6);
        assert!((N - Vector3::new(-sqrt2_inv, -sqrt2_inv, 0.0)).norm() < 1e-6);
        assert!((B - Vector3::new(0.0, 0.0, 1.0)).norm() < 1e-6);
    }

    #[test]
    fn test_contour_error_calculation() {
        let u = core::f64::consts::PI / 2.0; // Closest point on path is (0, 1, 0)
        let p_prime = circle_path_derivative(u);
        let p_double_prime = Vector3::new(-libm::cos(u), -libm::sin(u), 0.0);
        let frame = frenet_serret_frame(p_prime, p_double_prime);

        // Point is off the path in the normal direction
        let actual_pos = Point::new(0.0, 1.1, 0.0);
        let path_pos = circle_path(u);
        let pos_error = actual_pos - path_pos;

        let error = contour_error(pos_error, &frame);
        assert!((error - 0.1).abs() < 1e-6);
    }
}