//! Singularity-Consistent IK.

use crate::motion::kinematics::jacobian::Jacobian;
use libm::sqrtf;

pub struct SingularityConsistentIk<const N: usize> {
    jacobian_calculator: Jacobian<N>,
    manipulability_threshold: f32,
}

impl<const N: usize> SingularityConsistentIk<N> {
    pub fn new(screws: [crate::motion::kinematics::math::se3::Twist; N], manipulability_threshold: f32) -> Self {
        Self {
            jacobian_calculator: Jacobian::new(screws),
            manipulability_threshold,
        }
    }

    fn manipulability(&self, j: &[[f32; N]; 6]) -> f32 {
        let j_t = self.transpose(j);
        let jj_t = self.multiply(j, &j_t);
        self.determinant_6x6(&jj_t).sqrt()
    }

    pub fn solve(&self, thetas: &[f32; N], delta_x: &[f32; 6]) -> [f32; N] {
        let j = self.jacobian_calculator.jacobian(thetas);
        let w = self.manipulability(&j);

        if w < self.manipulability_threshold {
            // Lock primary rotary axis and use nullspace
            let j_pseudo_inv = self.damped_pseudo_inverse(&j, 0.1);
            let null_space_proj = self.null_space_projector(&j, &j_pseudo_inv);
            let q_null = self.calculate_q_null(thetas);

            let delta_theta_dls = self.multiply_Nx6_6x1(&j_pseudo_inv, delta_x);
            let delta_theta_null = self.multiply_NxN_Nx1(&null_space_proj, &q_null);

            let mut delta_theta = [0.0; N];
            for i in 0..N {
                delta_theta[i] = delta_theta_dls[i] + delta_theta_null[i];
            }
            delta_theta
        } else {
            let j_pseudo_inv = self.damped_pseudo_inverse(&j, 0.0);
            self.multiply_Nx6_6x1(&j_pseudo_inv, delta_x)
        }
    }

    fn damped_pseudo_inverse(&self, j: &[[f32; N]; 6], lambda: f32) -> [[f32; 6]; N] {
        let j_t = self.transpose(j);
        let jj_t = self.multiply(j, &j_t);
        let mut damped = jj_t;
        for i in 0..6 {
            damped[i][i] += lambda * lambda;
        }
        let inv = self.inverse_6x6(&damped).unwrap(); // Should handle error
        self.multiply_Nx6_6x6(&j_t, &inv)
    }

    fn null_space_projector(&self, j: &[[f32; N]; 6], j_pseudo_inv: &[[f32; 6]; N]) -> [[f32; N]; N] {
        let j_inv_j = self.multiply_Nx6_6xN(j_pseudo_inv, j);
        let mut p = [[0.0; N]; N];
        for i in 0..N {
            for j in 0..N {
                p[i][j] = if i == j { 1.0 } else { 0.0 } - j_inv_j[i][j];
            }
        }
        p
    }

    fn calculate_q_null(&self, thetas: &[f32; N]) -> [f32; N] {
        // Example: keep joint 4 close to 0
        let mut q_null = [0.0; N];
        q_null[3] = -0.1 * thetas[3];
        q_null
    }

    // Matrix utility functions (transpose, multiply, etc.) would be here.
    // These are duplicated in other files and should be moved to a common math module.
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

    fn determinant_6x6(&self, m: &[[f32; 6]; 6]) -> f32 { 1.0 }
    fn inverse_6x6(&self, m: &[[f32; 6]; 6]) -> Option<[[f32; 6]; 6]> { Some(*m) }
    fn multiply_Nx6_6x6(&self, a: &[[f32; 6]; N], b: &[[f32; 6]; 6]) -> [[f32; 6]; N] { [[0.0; 6]; N] }
    fn multiply_Nx6_6x1(&self, a: &[[f32; 6]; N], b: &[f32; 6]) -> [f32; N] { [0.0; N] }
    fn multiply_NxN_Nx1(&self, a: &[[f32; N]; N], b: &[f32; N]) -> [f32; N] { [0.0; N] }
    fn multiply_Nx6_6xN(&self, a: &[[f32; 6]; N], b: &[[f32; N]; 6]) -> [[f32; N]; N] { [[0.0; N]; N] }
}
