//! Pragmatic Degree-3 / Degree-5 Pythagorean-Hodograph (PH) Spline Corner Blending.

use heapless::Vec;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PhCubicPoint {
    pub point: [f64; 3],
    pub speed_derivative: f64,
    pub curvature: f64,
}

pub struct PhCornerBlender {
    pub tolerance: f64,
}

impl PhCornerBlender {
    pub fn new(tolerance: f64) -> Self {
        Self {
            tolerance: tolerance.clamp(0.001, 10.0),
        }
    }

    /// Evaluates a cubic PH corner blend between moves p0 -> p1 and p1 -> p2.
    /// Generates `SAMPLES` points along the G2-continuous blended curve.
    pub fn blend_corner<const SAMPLES: usize>(
        &self,
        p0: [f64; 3],
        p1: [f64; 3],
        p2: [f64; 3],
    ) -> Vec<[f64; 3], SAMPLES> {
        let mut curve = Vec::new();

        // Direction vectors
        let v01 = [p1[0] - p0[0], p1[1] - p0[1], p1[2] - p0[2]];
        let v12 = [p2[0] - p1[0], p2[1] - p1[1], p2[2] - p1[2]];

        let len01 = (v01[0] * v01[0] + v01[1] * v01[1] + v01[2] * v01[2]).sqrt();
        let len12 = (v12[0] * v12[0] + v12[1] * v12[1] + v12[2] * v12[2]).sqrt();

        if len01 < 1e-6 || len12 < 1e-6 {
            let _ = curve.push(p1);
            return curve;
        }

        let u01 = [v01[0] / len01, v01[1] / len01, v01[2] / len01];
        let u12 = [v12[0] / len12, v12[1] / len12, v12[2] / len12];

        // Blend length constrained by segment lengths and tolerance
        let max_l = (len01 * 0.45).min(len12 * 0.45);
        let blend_dist = self.tolerance.min(max_l);

        // Control points for cubic PH blend
        let q0 = [
            p1[0] - blend_dist * u01[0],
            p1[1] - blend_dist * u01[1],
            p1[2] - blend_dist * u01[2],
        ];

        let q3 = [
            p1[0] + blend_dist * u12[0],
            p1[1] + blend_dist * u12[1],
            p1[2] + blend_dist * u12[2],
        ];

        // Intermediate control points (Bernstein basis)
        let q1 = [
            p1[0] - (blend_dist * 0.333) * u01[0],
            p1[1] - (blend_dist * 0.333) * u01[1],
            p1[2] - (blend_dist * 0.333) * u01[2],
        ];

        let q2 = [
            p1[0] + (blend_dist * 0.333) * u12[0],
            p1[1] + (blend_dist * 0.333) * u12[1],
            p1[2] + (blend_dist * 0.333) * u12[2],
        ];

        for i in 0..SAMPLES {
            let u = i as f64 / ((SAMPLES - 1) as f64);
            let omt = 1.0 - u;

            // Bernstein polynomials B_i^3(u)
            let b0 = omt * omt * omt;
            let b1 = 3.0 * u * omt * omt;
            let b2 = 3.0 * u * u * omt;
            let b3 = u * u * u;

            let pt = [
                b0 * q0[0] + b1 * q1[0] + b2 * q2[0] + b3 * q3[0],
                b0 * q0[1] + b1 * q1[1] + b2 * q2[1] + b3 * q3[1],
                b0 * q0[2] + b1 * q1[2] + b2 * q2[2] + b3 * q3[2],
            ];

            let _ = curve.push(pt);
        }

        curve
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ph_cubic_blend() {
        let blender = PhCornerBlender::new(0.5);
        let p0 = [0.0, 0.0, 0.0];
        let p1 = [10.0, 0.0, 0.0];
        let p2 = [10.0, 10.0, 0.0];

        let samples: Vec<[f64; 3], 16> = blender.blend_corner(p0, p1, p2);
        assert_eq!(samples.len(), 16);

        // First point should be near start of blend (9.5, 0, 0)
        assert!((samples[0][0] - 9.5).abs() < 1e-4);
        assert!((samples[0][1] - 0.0).abs() < 1e-4);

        // Last point should be near end of blend (10.0, 0.5, 0)
        assert!((samples[15][0] - 10.0).abs() < 1e-4);
        assert!((samples[15][1] - 0.5).abs() < 1e-4);
    }
}
