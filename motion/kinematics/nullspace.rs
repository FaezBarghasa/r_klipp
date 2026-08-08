//! Nullspace projection for secondary objectives.

use crate::motion::kinematics::jacobian::Jacobian;

pub struct NullspaceProjector<const N: usize> {
    jacobian_calculator: Jacobian<N>,
}

impl<const N: usize> NullspaceProjector<N> {
    pub fn new(screws: [crate::motion::kinematics::math::se3::Twist; N]) -> Self {
        Self {
            jacobian_calculator: Jacobian::new(screws),
        }
    }

    pub fn project(&self, thetas: &[f32; N], secondary_goal: &[f32; N]) -> [f32; N] {
        let j = self.jacobian_calculator.jacobian(thetas);
        let j_inv = self.pseudo_inverse(&j);
        let j_inv_j = self.multiply_Nx6_6xN(&j_inv, &j);
        let p = self.subtract_identity(&j_inv_j);
        self.multiply_NxN_Nx1(&p, secondary_goal)
    }

    fn pseudo_inverse(&self, m: &[[f32; N]; 6]) -> [[f32; N]; 6] {
        // Placeholder for a proper pseudo-inverse calculation (e.g., using SVD).
        [[0.0; N]; 6]
    }

    fn multiply_Nx6_6xN(&self, a: &[[f32; N]; 6], b: &[[f32; N]; 6]) -> [[f32; N]; N] {
        let mut res = [[0.0; N]; N];
        for i in 0..N {
            for j in 0..N {
                for k in 0..6 {
                    res[i][j] += a[k][i] * b[k][j]; // Transposing 'a'
                }
            }
        }
        res
    }

    fn subtract_identity(&self, m: &[[f32; N]; N]) -> [[f32; N]; N] {
        let mut res = *m;
        for i in 0..N {
            res[i][i] = 1.0 - res[i][i];
        }
        res
    }

    fn multiply_NxN_Nx1(&self, a: &[[f32; N]; N], b: &[f32; N]) -> [f32; N] {
        let mut res = [0.0; N];
        for i in 0..N {
            for j in 0..N {
                res[i] += a[i][j] * b[j];
            }
        }
        res
    }
}
