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

/// An Extended Kalman Filter for disturbance estimation.
pub struct DisturbanceObserver<const N_STATES: usize, const N_INPUTS: usize, const N_MEAS: usize> {
    // State vector [position, velocity, disturbance_force]
    pub x: Vector<N_STATES>,
    // Covariance matrix
    pub P: Matrix<N_STATES, N_STATES>,
    // Process noise covariance
    pub Q: Matrix<N_STATES, N_STATES>,
    // Measurement noise covariance
    pub R: Matrix<N_MEAS, N_MEAS>,
}

impl<const NS: usize, const NI: usize, const NM: usize> DisturbanceObserver<NS, NI, NM> {
    pub fn predict<F, FJacobian>(&mut self, u: &Vector<NI>, f: F, F_jacobian: FJacobian)
    where
        F: Fn(&Vector<NS>, &Vector<NI>) -> Vector<NS>,
        FJacobian: Fn(&Vector<NS>, &Vector<NI>) -> Matrix<NS, NS>,
    {
        let F_mat = F_jacobian(&self.x, u);
        self.x = f(&self.x, u);
        self.P = F_mat * self.P * F_mat.transpose() + self.Q;
    }

    pub fn update<H, HJacobian>(&mut self, z: &Vector<NM>, h: H, H_jacobian: HJacobian)
    where
        H: Fn(&Vector<NS>) -> Vector<NM>,
        HJacobian: Fn(&Vector<NS>) -> Matrix<NM, NS>,
    {
        let H_mat = H_jacobian(&self.x);
        let y = *z - h(&self.x);
        let S = H_mat * self.P * H_mat.transpose() + self.R;
        let S_inv = S.try_inverse().expect("EKF update failed: S is not invertible");
        let K = self.P * H_mat.transpose() * S_inv;
        self.x += K * y;
        self.P = (Matrix::<NS, NS>::identity() - K * H_mat) * self.P;
    }

    pub fn get_disturbance(&self) -> f64 {
        // Assuming disturbance is the last state
        self.x[NS - 1]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Simple 1D system: x = [pos, vel, disturbance_force]
    const N_STATES: usize = 3;
    const N_INPUTS: usize = 1;
    const N_MEAS: usize = 1;

    // State transition function
    fn f(x: &Vector<N_STATES>, u: &Vector<N_INPUTS>) -> Vector<N_STATES> {
        let dt = 0.001; // 1kHz
        let mass = 1.0;
        let mut x_new = *x;
        x_new[0] += x[1] * dt; // pos += vel * dt
        x_new[1] += (u[0] - x[2]) / mass * dt; // vel += (force - disturbance) / mass * dt
        // disturbance is assumed to be a random walk
        x_new
    }

    // Jacobian of f
    fn F_jacobian(_x: &Vector<N_STATES>, _u: &Vector<N_INPUTS>) -> Matrix<N_STATES, N_STATES> {
        let dt = 0.001;
        let mass = 1.0;
        Matrix::<N_STATES, N_STATES>::from_row_slice(&[
            1.0, dt, 0.0,
            0.0, 1.0, -dt / mass,
            0.0, 0.0, 1.0, // Random walk model for disturbance
        ])
    }

    // Measurement function (we only measure position)
    fn h(x: &Vector<N_STATES>) -> Vector<N_MEAS> {
        Vector::<N_MEAS>::from_row_slice(&[x[0]])
    }

    // Jacobian of h
    fn H_jacobian(_x: &Vector<N_STATES>) -> Matrix<N_MEAS, N_STATES> {
        Matrix::<N_MEAS, N_STATES>::from_row_slice(&[1.0, 0.0, 0.0])
    }

    #[test]
    fn test_ekf_disturbance_estimation() {
        let mut ekf = DisturbanceObserver {
            x: Vector::<N_STATES>::zero(),
            P: Matrix::<N_STATES, N_STATES>::identity(),
            Q: Matrix::<N_STATES, N_STATES>::from_diagonal(&Vector::<N_STATES>::from_row_slice(&[1e-6, 1e-4, 1e-2])),
            R: Matrix::<N_MEAS, N_MEAS>::from_diagonal(&Vector::<N_MEAS>::from_row_slice(&[1e-4])),
        };

        let true_disturbance = 5.0;
        let mut true_pos = 0.0;
        let mut true_vel = 0.0;
        let dt = 0.001;
        let mass = 1.0;

        for _ in 0..1000 {
            let u = Vector::<N_INPUTS>::from_row_slice(&[10.0]); // Apply constant force

            // Simulate true system
            true_vel += (u[0] - true_disturbance) / mass * dt;
            true_pos += true_vel * dt;
            let z = Vector::<N_MEAS>::from_row_slice(&[true_pos]); // Measurement with no noise for simplicity

            ekf.predict(&u, f, F_jacobian);
            ekf.update(&z, h, H_jacobian);
        }

        let estimated_disturbance = ekf.get_disturbance();
        assert!((estimated_disturbance - true_disturbance).abs() < 0.1, "Disturbance estimation failed");
    }
}