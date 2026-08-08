
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

use crate::interpolation::nurbs_realtime::Nurbs;

const NR_MAX_ITERATIONS: usize = 3;
const NR_TOLERANCE: f64 = 1e-5;

impl<const DEGREE: usize, const N_CONTROL_POINTS: usize> Nurbs<DEGREE, N_CONTROL_POINTS> {
    /// Corrects the parameter u to match the target arc length using Newton-Raphson.
    pub fn correct_for_arc_length(
        &self,
        mut u: f64,
        target_arc_length: f64,
        initial_u: f64,
    ) -> Result<f64, &'static str> {
        for _ in 0..NR_MAX_ITERATIONS {
            let current_arc_length = self.arc_length(initial_u, u);
            let error = current_arc_length - target_arc_length;

            if error.abs() < NR_TOLERANCE {
                return Ok(u);
            }

            let C_prime_norm = self.derivative(u).norm();
            if C_prime_norm < 1e-9 {
                return Err("Derivative is zero, cannot correct arc length.");
            }

            u -= error / C_prime_norm;
        }
        Err("Newton-Raphson failed to converge for arc length correction.")
    }

    /// Calculates the arc length of the curve from u0 to u1 using numerical integration.
    fn arc_length(&self, u0: f64, u1: f64) -> f64 {
        // Simple numerical integration (e.g., Simpson's rule) would be used here.
        // For simplicity, we use a basic trapezoidal rule.
        let n_steps = 100;
        let mut length = 0.0;
        let du = (u1 - u0) / n_steps as f64;
        for i in 0..n_steps {
            let u_i = u0 + i as f64 * du;
            let u_i1 = u0 + (i + 1) as f64 * du;
            let p1_norm = self.derivative(u_i).norm();
            let p2_norm = self.derivative(u_i1).norm();
            length += (p1_norm + p2_norm) / 2.0 * du;
        }
        length
    }
}
