//! Newton-Raphson feedback loop for arc length correction.

use crate::motion::interpolation::nurbs_realtime::NurbsRealtime;
use libm::sqrtf;

pub struct NewtonRaphson<const DEGREE: usize, const CTRL_PTS: usize> {
    nurbs: NurbsRealtime<DEGREE, CTRL_PTS>,
    max_iter: u32,
    tolerance: f32,
}

impl<const DEGREE: usize, const CTRL_PTS: usize> NewtonRaphson<DEGREE, CTRL_PTS> {
    pub fn new(nurbs: NurbsRealtime<DEGREE, CTRL_PTS>, max_iter: u32, tolerance: f32) -> Self {
        Self { nurbs, max_iter, tolerance }
    }

    fn arc_length(&self, u: f32) -> f32 {
        // A proper implementation would use numerical integration (e.g., Gaussian quadrature).
        // This is a simplified placeholder.
        let ders = self.nurbs.point_and_ders(u, 1);
        let norm = sqrtf(ders[1][0]*ders[1][0] + ders[1][1]*ders[1][1] + ders[1][2]*ders[1][2]);
        norm * u
    }

    pub fn correct(&self, mut u: f32, s_target: f32) -> Result<f32, &'static str> {
        for _ in 0..self.max_iter {
            let s_current = self.arc_length(u);
            let error = s_current - s_target;

            if error.abs() < self.tolerance {
                return Ok(u);
            }

            let ders = self.nurbs.point_and_ders(u, 1);
            let norm = sqrtf(ders[1][0]*ders[1][0] + ders[1][1]*ders[1][1] + ders[1][2]*ders[1][2]);

            if norm.abs() < 1e-6 {
                return Err("Zero derivative");
            }

            u -= error / norm;
        }

        Err("Newton-Raphson failed to converge")
    }
}
