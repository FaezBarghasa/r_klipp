//! Damped Least Squares (DLS) Inverse Kinematics.

use crate::motion::kinematics::math::se3::{Transform, Twist};
use crate::motion::kinematics::jacobian::Jacobian;
use crate::motion::kinematics::poe_fk::PoeKinematics;

pub struct DlsIk<const N: usize> {
    kinematics: PoeKinematics<N>,
    jacobian_calculator: Jacobian<N>,
    lambda: f32,
    max_iter: u32,
    tolerance: f32,
}

impl<const N: usize> DlsIk<N> {
    pub fn new(screws: [Twist; N], m: Transform, lambda: f32, max_iter: u32, tolerance: f32) -> Self {
        Self {
            kinematics: PoeKinematics::new(screws, m),
            jacobian_calculator: Jacobian::new(screws),
            lambda,
            max_iter,
            tolerance,
        }
    }

    pub fn solve(&self, target: &Transform, initial_thetas: &[f32; N]) -> Result<[f32; N], &'static str> {
        let mut thetas = *initial_thetas;
        let mut current_transform = self.kinematics.fk(&thetas);
        let mut error = self.calculate_error(target, &current_transform);

        for _ in 0..self.max_iter {
            if self.is_converged(&error) {
                return Ok(thetas);
            }

            let j = self.jacobian_calculator.jacobian(&thetas);
            let j_t = self.transpose(&j);
            let jj_t = self.multiply(&j, &j_t);
            let damped_jj_t = self.add_damping(&jj_t);
            let inv_damped_jj_t = self.inverse_6x6(&damped_jj_t).ok_or("Matrix not invertible")?;
            let j_pseudo_inv = self.multiply_6xN_6x6(&j_t, &inv_damped_jj_t);

            let delta_theta = self.multiply_Nx6_6x1(&j_pseudo_inv, &error);

            for i in 0..N {
                thetas[i] += delta_theta[i];
            }

            current_transform = self.kinematics.fk(&thetas);
            error = self.calculate_error(target, &current_transform);
        }

        Err("IK failed to converge")
    }

    fn calculate_error(&self, target: &Transform, current: &Transform) -> [f32; 6] {
        let mut error = [0.0; 6];
        let error_transform = *target * current.inverse();

        error[0] = error_transform.trans[0];
        error[1] = error_transform.trans[1];
        error[2] = error_transform.trans[2];

        // Simplified orientation error
        error[3] = (error_transform.rot[2][1] - error_transform.rot[1][2]) / 2.0;
        error[4] = (error_transform.rot[0][2] - error_transform.rot[2][0]) / 2.0;
        error[5] = (error_transform.rot[1][0] - error_transform.rot[0][1]) / 2.0;

        error
    }

    fn is_converged(&self, error: &[f32; 6]) -> bool {
        let mut norm = 0.0;
        for e in error.iter() {
            norm += e * e;
        }
        norm.sqrt() < self.tolerance
    }

    fn transpose(&self, m: &[[f32; N]; 6]) -> [[f32; 6]; N] {
        let mut res = [[0.0; 6]; N];
        for i in 0..N {
            for j in 0..6 {
                res[i][j] = m[j][i];
            }
        }
        res
    }

    fn multiply(&self, a: &[[f32; N]; 6], b: &[[f32; 6]; N]) -> [[f32; 6]; 6] {
        let mut res = [[0.0; 6]; 6];
        for i in 0..6 {
            for j in 0..6 {
                for k in 0..N {
                    res[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        res
    }

    fn add_damping(&self, m: &[[f32; 6]; 6]) -> [[f32; 6]; 6] {
        let mut res = *m;
        for i in 0..6 {
            res[i][i] += self.lambda * self.lambda;
        }
        res
    }

    fn inverse_6x6(&self, m: &[[f32; 6]; 6]) -> Option<[[f32; 6]; 6]> {
        // Using Gauss-Jordan elimination for simplicity.
        // A more robust implementation would use LU decomposition or SVD.
        let mut aug = [[0.0; 12]; 6];
        for i in 0..6 {
            for j in 0..6 {
                aug[i][j] = m[i][j];
            }
            aug[i][i+6] = 1.0;
        }

        for i in 0..6 {
            let mut pivot = i;
            for j in i+1..6 {
                if aug[j][i].abs() > aug[pivot][i].abs() {
                    pivot = j;
                }
            }
            aug.swap(i, pivot);

            let div = aug[i][i];
            if div.abs() < 1e-9 { return None; }

            for j in i..12 {
                aug[i][j] /= div;
            }

            for j in 0..6 {
                if i != j {
                    let mult = aug[j][i];
                    for k in i..12 {
                        aug[j][k] -= mult * aug[i][k];
                    }
                }
            }
        }

        let mut inv = [[0.0; 6]; 6];
        for i in 0..6 {
            for j in 0..6 {
                inv[i][j] = aug[i][j+6];
            }
        }
        Some(inv)
    }

    fn multiply_6xN_6x6(&self, a: &[[f32; 6]; N], b: &[[f32; 6]; 6]) -> [[f32; 6]; N] {
        let mut res = [[0.0; 6]; N];
        for i in 0..N {
            for j in 0..6 {
                for k in 0..6 {
                    res[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        res
    }

    fn multiply_Nx6_6x1(&self, a: &[[f32; 6]; N], b: &[f32; 6]) -> [f32; N] {
        let mut res = [0.0; N];
        for i in 0..N {
            for j in 0..6 {
                res[i] += a[i][j] * b[j];
            }
        }
        res
    }
}
