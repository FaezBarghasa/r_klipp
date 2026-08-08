
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

use crate::math::Point;

pub struct Nurbs<const DEGREE: usize, const N_CONTROL_POINTS: usize> {
    pub control_points: [Point; N_CONTROL_POINTS],
    pub weights: [f64; N_CONTROL_POINTS],
    pub knots: [f64; { N_CONTROL_POINTS + DEGREE + 1 }],
}

impl<const DEGREE: usize, const N_CONTROL_POINTS: usize> Nurbs<DEGREE, N_CONTROL_POINTS> {
    /// Evaluates the NURBS curve at parameter u.
    pub fn evaluate(&self, u: f64) -> Point {
        let mut numerator = Point::origin().to_vector();
        let mut denominator = 0.0;

        for i in 0..N_CONTROL_POINTS {
            let basis = self.basis_function(i, DEGREE, u);
            numerator = numerator + self.control_points[i].to_vector() * self.weights[i] * basis;
            denominator += self.weights[i] * basis;
        }

        (numerator / denominator).into()
    }

    /// Evaluates the first derivative of the NURBS curve.
    pub fn derivative(&self, u: f64) -> crate::math::Vector3 {
        // This is a simplified version. A full implementation is more complex.
        let p1 = self.evaluate(u);
        let p2 = self.evaluate(u + 1e-6);
        (p2 - p1) / 1e-6
    }

    /// Evaluates the second derivative of the NURBS curve.
    pub fn second_derivative(&self, u: f64) -> crate::math::Vector3 {
        // Simplified version
        let v1 = self.derivative(u);
        let v2 = self.derivative(u + 1e-6);
        (v2 - v1) / 1e-6
    }

    /// Cox-de Boor recursion for B-spline basis functions.
    fn basis_function(&self, i: usize, k: usize, u: f64) -> f64 {
        if k == 0 {
            if self.knots[i] <= u && u < self.knots[i + 1] {
                1.0
            } else {
                0.0
            }
        } else {
            let mut term1 = 0.0;
            if self.knots[i + k] - self.knots[i] != 0.0 {
                term1 = (u - self.knots[i]) / (self.knots[i + k] - self.knots[i])
                    * self.basis_function(i, k - 1, u);
            }

            let mut term2 = 0.0;
            if self.knots[i + k + 1] - self.knots[i + 1] != 0.0 {
                term2 = (self.knots[i + k + 1] - u) / (self.knots[i + k + 1] - self.knots[i + 1])
                    * self.basis_function(i + 1, k - 1, u);
            }
            term1 + term2
        }
    }

    /// Updates the NURBS parameter u using a second-order Taylor series expansion.
    pub fn update_parameter_taylor(&self, u: f64, v: f64, dt: f64) -> f64 {
        let C_prime = self.derivative(u);
        let C_prime_norm = C_prime.norm();
        if C_prime_norm < 1e-9 {
            return u;
        }

        let C_double_prime = self.second_derivative(u);
        let term1 = v * dt / C_prime_norm;
        let term2 = (dt * dt / 2.0)
            * (C_prime.dot(C_double_prime) / C_prime_norm.powi(4))
            * v.powi(2);

        u + term1 - term2
    }
}
