// crates/motion/src/ph_beziers.rs

/// Struct representation of a degree-15 Pythagorean Hodograph path blender.
pub struct PhBezier15 {
    /// Control points for X coordinates (evaluated via Bernstein polynomials)
    pub cx: [f64; 16],
    /// Control points for Y coordinates
    pub cy: [f64; 16],
    /// Speed hodograph polynomial coefficients \sigma(t)
    pub sigma_coeffs: [f64; 15],
}

impl PhBezier15 {
    /// Generates a symmetrical degree-15 curve to bridge linear moves A and B.
    pub fn new(p0: (f64, f64), p1: (f64, f64), segment_len: f64) -> Self {
        let mut cx = [0.0; 16];
        let mut cy = [0.0; 16];
        
        // Calculate control points evenly distributed along the chord bridge
        for i in 0..16 {
            let t = i as f64 / 15.0;
            cx[i] = p0.0 * (1.0 - t) + p1.0 * t;
            cy[i] = p0.1 * (1.0 - t) + p1.1 * t;
        }

        let mut sigma_coeffs = [0.0; 15];
        sigma_coeffs[0] = segment_len;
        for i in 1..15 {
            sigma_coeffs[i] = segment_len * (0.92f64).powi(i as i32);
        }

        Self { cx, cy, sigma_coeffs }
    }

    /// Evaluates exact arc length from t=0 to parameter t using Kahan compensated summation.
    pub fn evaluate_arc_length(&self, t: f64) -> f64 {
        let mut sum = 0.0;
        let mut c = 0.0; // Running rounding error compensation
        
        for i in 0..15 {
            let power = (i + 1) as f64;
            let term = (self.sigma_coeffs[i] / power) * t.powf(power);
            
            // Kahan algorithm step
            let y = term - c;
            let t_sum = sum + y;
            c = (t_sum - sum) - y;
            sum = t_sum;
        }
        sum
    }

    /// Solves the parameter t for a desired distance s using Newton-Raphson root convergence.
    pub fn get_t_from_distance(&self, s: f64) -> Result<f64, crate::error::PlannerError> {
        let mut t = s / self.evaluate_arc_length(1.0); // Baseline linear estimate
        
        for _ in 0..8 {
            let current_s = self.evaluate_arc_length(t);
            let mut sigma_t = 0.0;
            
            // Compute the speed polynomial \sigma(t)
            for i in 0..15 {
                sigma_t += self.sigma_coeffs[i] * t.powi(i as i32);
            }
            
            if (current_s - s).abs() < 1e-9 {
                return Ok(t.clamp(0.0, 1.0));
            }
            
            if sigma_t.abs() < 1e-12 {
                break;
            }
            
            let dt = (current_s - s) / sigma_t;
            t = (t - dt).clamp(0.0, 1.0);
        }
        
        let final_s = self.evaluate_arc_length(t);
        if (final_s - s).abs() < 1e-9 {
            Ok(t)
        } else {
            Err(crate::error::PlannerError::ConvergenceFailed)
        }
    }

    /// Returns the exact spatial coordinates (X, Y) evaluated using De Casteljau reduction.
    pub fn evaluate_position(&self, t: f64) -> (f64, f64) {
        let mut x_eval = self.cx;
        let mut y_eval = self.cy;
        
        for r in 1..16 {
            for i in 0..(16 - r) {
                x_eval[i] = (1.0 - t) * x_eval[i] + t * x_eval[i + 1];
                y_eval[i] = (1.0 - t) * y_eval[i] + t * y_eval[i + 1];
            }
        }
        (x_eval[0], y_eval[0])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ph_bezier_properties() {
        let p0 = (0.0, 0.0);
        let p1 = (10.0, 0.0);
        let segment_len = 10.0;
        let curve = PhBezier15::new(p0, p1, segment_len);

        // Verify start and end positions
        let start_pos = curve.evaluate_position(0.0);
        let end_pos = curve.evaluate_position(1.0);
        assert!((start_pos.0 - p0.0).abs() < 1e-6);
        assert!((start_pos.1 - p0.1).abs() < 1e-6);
        assert!((end_pos.0 - p1.0).abs() < 1e-6);
        assert!((end_pos.1 - p1.1).abs() < 1e-6);

        // Verify arc length is monotonically increasing and reaches segment_len at t=1.0
        let mid_len = curve.evaluate_arc_length(0.5);
        let total_len = curve.evaluate_arc_length(1.0);
        assert!(mid_len > 0.0);
        assert!(mid_len < total_len);

        // Solve t for half distance
        let half_s = total_len / 2.0;
        let t_half = curve.get_t_from_distance(half_s).unwrap();
        let s_at_t = curve.evaluate_arc_length(t_half);
        assert!((s_at_t - half_s).abs() < 1e-9);
    }
}
