//! C^3 continuous (snap-limited) feedrate scheduling.

use crate::motion::interpolation::nurbs_realtime::NurbsRealtime;
use libm::sqrtf;

pub struct FeedrateC3<const DEGREE: usize, const CTRL_PTS: usize> {
    nurbs: NurbsRealtime<DEGREE, CTRL_PTS>,
    max_snap: f32,
    chord_error_limit: f32,
}

impl<const DEGREE: usize, const CTRL_PTS: usize> FeedrateC3<DEGREE, CTRL_PTS> {
    pub fn new(nurbs: NurbsRealtime<DEGREE, CTRL_PTS>, max_snap: f32, chord_error_limit: f32) -> Self {
        Self { nurbs, max_snap, chord_error_limit }
    }

    fn curvature(&self, u: f32) -> f32 {
        let ders = self.nurbs.point_and_ders(u, 2);
        let c_prime = &ders[1];
        let c_double_prime = &ders[2];

        let cross_product = [
            c_prime[1] * c_double_prime[2] - c_prime[2] * c_double_prime[1],
            c_prime[2] * c_double_prime[0] - c_prime[0] * c_double_prime[2],
            c_prime[0] * c_double_prime[1] - c_prime[1] * c_double_prime[0],
        ];

        let cross_norm = sqrtf(cross_product[0]*cross_product[0] + cross_product[1]*cross_product[1] + cross_product[2]*cross_product[2]);
        let c_prime_norm = sqrtf(c_prime[0]*c_prime[0] + c_prime[1]*c_prime[1] + c_prime[2]*c_prime[2]);

        cross_norm / (c_prime_norm * c_prime_norm * c_prime_norm)
    }

    pub fn schedule(&self, u: f32, current_feedrate: f32) -> f32 {
        let k = self.curvature(u);
        let feedrate_limit_from_curvature = if k > 1e-6 {
            sqrtf(self.chord_error_limit / k)
        } else {
            f32::MAX
        };

        // This is a simplified snap-limited profile. A full implementation
        // would involve a more complex 9-segment profile.
        let feedrate_limit_from_snap = (self.max_snap / (k + 1e-6)).powf(1.0/3.0);

        current_feedrate.min(feedrate_limit_from_curvature).min(feedrate_limit_from_snap)
    }
}
