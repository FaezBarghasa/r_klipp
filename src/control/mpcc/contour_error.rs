
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

/// Calculates the Frenet-Serret frame (T, N, B) for a given path.
pub fn frenet_serret_frame(
    path_derivative: Vector3,
    path_second_derivative: Vector3,
) -> (Vector3, Vector3, Vector3) {
    let T = path_derivative.normalize();
    let B = path_derivative.cross(path_second_derivative).normalize();
    let N = B.cross(T);
    (T, N, B)
}

/// Calculates the contour error by projecting the position error onto the Frenet-Serret frame.
pub fn contour_error(
    position_error: Vector3,
    frenet_frame: &(Vector3, Vector3, Vector3),
) -> (f64, f64) {
    let (_T, N, B) = frenet_frame;
    let error_normal = position_error.dot(*N);
    let error_binormal = position_error.dot(*B);
    (error_normal, error_binormal)
}

/// Finds the closest point on a path (represented as a function of u) to a given point.
pub fn find_closest_point_on_path<F>(
    path_evaluator: F,
    point: Point,
    initial_u: f64,
) -> f64
where
    F: Fn(f64) -> Point,
{
    let mut u = initial_u;
    for _ in 0..5 { // Newton-Raphson iterations
        let p = path_evaluator(u);
        let p_prime = (path_evaluator(u + 1e-6) - p) / 1e-6;
        let error = p - point;
        u -= error.dot(p_prime) / p_prime.dot(p_prime);
    }
    u
}
