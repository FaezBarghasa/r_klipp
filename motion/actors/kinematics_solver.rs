//! Async actor for solving kinematics.

use crate::motion::kinematics::dls_ik::DlsIk;
use crate::motion::kinematics::math::se3::{Transform, Twist};

pub enum KinematicError {
    Unreachable,
    Singularity,
}

pub struct KinematicsSolver<const N: usize> {
    ik_solver: DlsIk<N>,
}

impl<const N: usize> KinematicsSolver<N> {
    pub fn new(screws: [Twist; N], m: Transform, lambda: f32, max_iter: u32, tolerance: f32) -> Self {
        Self {
            ik_solver: DlsIk::new(screws, m, lambda, max_iter, tolerance),
        }
    }

    pub async fn solve(&self, target: Transform, initial_thetas: [f32; N]) -> Result<[f32; N], KinematicError> {
        // In a real async environment, this would be a true async call.
        // For now, we just wrap the synchronous solver.
        self.ik_solver.solve(&target, &initial_thetas).map_err(|_| KinematicError::Unreachable)
    }
}
