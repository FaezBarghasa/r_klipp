
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

use crate::math::{Matrix, Vector};

/// A simple Extended Kalman Filter for disturbance estimation.
pub struct DisturbanceObserver<const N_STATES: usize, const N_INPUTS: usize, const N_MEAS: usize> {
    // State vector [position, velocity, disturbance_force]
    pub x: Vector<N_STATES>,
    // Covariance matrix
    pub P: Matrix<N_STATES, N_STATES>,
    // Process noise covariance
    pub Q: Matrix<N_STATES, N_STATES>,
    // Measurement noise covariance
    pub R: Matrix<N_MEAS, N_MEAS>,
    // State transition model
    pub F: Matrix<N_STATES, N_STATES>,
    // Input model
    pub B: Matrix<N_STATES, N_INPUTS>,
    // Measurement model
    pub H: Matrix<N_MEAS, N_STATES>,
}

impl<const NS: usize, const NI: usize, const NM: usize> DisturbanceObserver<NS, NI, NM> {
    pub fn predict(&mut self, u: &Vector<NI>) {
        self.x = self.F * self.x + self.B * *u;
        self.P = self.F * self.P * self.F.transpose() + self.Q;
    }

    pub fn update(&mut self, z: &Vector<NM>) {
        let y = *z - self.H * self.x;
        let S = self.H * self.P * self.H.transpose() + self.R;
        let K = self.P * self.H.transpose() * S.inverse().unwrap();
        self.x = self.x + K * y;
        self.P = (Matrix::<NS, NS>::identity() - K * self.H) * self.P;
    }

    pub fn get_disturbance(&self) -> f64 {
        // Assuming disturbance is the last state
        self.x[NS - 1]
    }
}
