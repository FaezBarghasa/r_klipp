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
use heapless::Vec as HeaplessVec;

const QP_MAX_ITERATIONS: usize = 20;
const QP_TOLERANCE: f64 = 1e-6;

/// Solves a quadratic program of the form: min 0.5 * x' * H * x + g' * x
/// subject to A * x = b, C * x >= d
/// using an active-set method.
pub fn solve_qp<const N: usize, const ME: usize, const MI: usize>(
    H: &Matrix<N, N>,
    g: &Vector<N>,
    A: &Matrix<ME, N>,
    b: &Vector<ME>,
    C: &Matrix<MI, N>,
    d: &Vector<MI>,
) -> Result<Vector<N>, &'static str> {
    let mut x = Vector::<N>::zero();
    let mut active_set: HeaplessVec<usize, MI> = HeaplessVec::new();

    for _ in 0..QP_MAX_ITERATIONS {
        let grad = *H * x + *g;
        let mut lagrange_mult = Vector::<MI>::zero();
        let mut is_optimal = true;

        // Solve KKT system for current active set
        let n_active = active_set.len();
        if n_active + ME > 0 {
            let mut kkt_mat = Matrix::<{ N + ME + MI }, { N + ME + MI }>::zero();
            kkt_mat.view_mut((0, 0), (N, N)).copy_from(H);

            let mut C_active = Matrix::<MI, N>::zero();
            for (i, &idx) in active_set.iter().enumerate() {
                kkt_mat.view_mut((N + ME + i, 0), (1, N)).copy_from(&C.row(idx));
                kkt_mat.view_mut((0, N + ME + i), (N, 1)).copy_from(&C.row(idx).transpose());
                C_active.set_row(i, &C.row(idx));
            }
            if ME > 0 {
                kkt_mat.view_mut((N, 0), (ME, N)).copy_from(A);
                kkt_mat.view_mut((0, N), (N, ME)).copy_from(&A.transpose());
            }

            let kkt_inv = kkt_mat.try_inverse().ok_or("KKT matrix not invertible")?;

            let mut rhs = Vector::<{ N + ME + MI }>::zero();
            rhs.view_mut((0, 0), (N, 1)).copy_from(&(-grad));

            let kkt_sol = kkt_inv * rhs;
            let p = kkt_sol.rows(0, N);

            x += p;

            // Check for blocking constraints
            let mut alpha = 1.0;
            let mut blocking_constraint = None;
            for i in 0..MI {
                if !active_set.contains(&i) {
                    let C_i = C.row(i);
                    let denominator = C_i * p;
                    if denominator < -QP_TOLERANCE {
                        let numerator = d[i] - C_i * x;
                        let t = numerator / denominator;
                        if t < alpha {
                            alpha = t;
                            blocking_constraint = Some(i);
                        }
                    }
                }
            }
            x += p * alpha;

            if let Some(idx) = blocking_constraint {
                active_set.push(idx).map_err(|_| "Active set full")?;
                is_optimal = false;
            }

            // Check Lagrange multipliers for optimality
            let lagrange_all = kkt_sol.rows(N, ME + MI);
            for (i, &idx) in active_set.iter().enumerate() {
                lagrange_mult[idx] = lagrange_all[ME + i];
            }

            if is_optimal {
                let mut min_lambda = f64::MAX;
                let mut min_idx = None;
                for (i, &idx) in active_set.iter().enumerate() {
                    if lagrange_mult[idx] < min_lambda {
                        min_lambda = lagrange_mult[idx];
                        min_idx = Some(i);
                    }
                }

                if min_lambda >= -QP_TOLERANCE {
                    return Ok(x); // Optimal solution found
                } else if let Some(idx_to_remove) = min_idx {
                    active_set.remove(idx_to_remove);
                }
            }
        } else {
            // Unconstrained step
            let H_inv = H.try_inverse().ok_or("H is not invertible")?;
            let p = H_inv * (-grad);
            x += p;
        }
    }

    Err("QP solver did not converge")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unconstrained_qp() {
        let H = Matrix::<2, 2>::from_row_slice(&[1.0, 0.0, 0.0, 1.0]);
        let g = Vector::<2>::from_row_slice(&[1.0, 1.0]);
        let A = Matrix::<0, 2>::zero();
        let b = Vector::<0>::zero();
        let C = Matrix::<0, 2>::zero();
        let d = Vector::<0>::zero();

        let x = solve_qp(&H, &g, &A, &b, &C, &d).unwrap();
        assert!((x[0] - -1.0).abs() < 1e-6);
        assert!((x[1] - -1.0).abs() < 1e-6);
    }
}