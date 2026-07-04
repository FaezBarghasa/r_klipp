
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

pub struct C3FeedrateScheduler {
    pub max_velocity: f64,
    pub max_acceleration: f64,
    pub max_jerk: f64,
    pub max_snap: f64,
    pub chord_error_limit: f64,
}

impl C3FeedrateScheduler {
    /// Calculates the maximum feedrate at a point on the curve based on curvature.
    pub fn limit_feedrate_for_curvature<const D: usize, const N: usize>(
        &self,
        nurbs: &Nurbs<D, N>,
        u: f64,
    ) -> f64 {
        let C_prime = nurbs.derivative(u);
        let C_double_prime = nurbs.second_derivative(u);
        let curvature = C_prime.cross(C_double_prime).norm() / C_prime.norm().powi(3);

        if curvature < 1e-9 {
            self.max_velocity
        } else {
            let v_limit_from_accel = libm::sqrt(self.max_acceleration / curvature);
            let v_limit_from_chord_error = libm::sqrt(8.0 * self.chord_error_limit / curvature);
            v_limit_from_accel.min(v_limit_from_chord_error).min(self.max_velocity)
        }
    }

    // A full 9-segment snap-limited profile is very complex.
    // This is a simplified placeholder showing the concept.
    pub fn generate_profile(&self, distance: f64) -> (f64, f64, f64, f64) {
        // This would involve solving a system of equations to find the time intervals
        // for each of the 9 segments of the snap-limited profile.
        // For now, we return dummy values.
        let time = distance / (self.max_velocity * 0.8);
        let v = self.max_velocity * 0.8;
        let a = self.max_acceleration * 0.5;
        let j = self.max_jerk * 0.5;
        (time, v, a, j)
    }
}
