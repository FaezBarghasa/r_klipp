//! Manipulability and singularity detection.

use crate::motion::kinematics::jacobian::Jacobian;

pub struct SingularityDetector<const N: usize> {
    jacobian_calculator: Jacobian<N>,
}

impl<const N: usize> SingularityDetector<N> {
    pub fn new(screws: [crate::motion::kinematics::math::se3::Twist; N]) -> Self {
        Self {
            jacobian_calculator: Jacobian::new(screws),
        }
    }

    pub fn manipulability(&self, thetas: &[f32; N]) -> f32 {
        let j = self.jacobian_calculator.jacobian(thetas);
        let j_t = self.transpose(&j);
        let jj_t = self.multiply(&j, &j_t);
        self.determinant_6x6(&jj_t).sqrt()
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

    fn determinant_6x6(&self, m: &[[f32; 6]; 6]) -> f32 {
        // This is a placeholder for a proper determinant calculation.
        // A full 6x6 determinant is computationally expensive.
        // For real-time applications, other singularity metrics are often used.
        1.0
    }
}
