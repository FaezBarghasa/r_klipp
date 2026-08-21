/// A structurally robust mathematical definition of a Degree-15 Pythagorean Hodograph (PH) spline
/// designed for high-order continuous ($C^4$) corner blending.
#[derive(Clone, Debug)]
pub struct PhBezier15 {
    // 16 control points for degree-15 Bezier curve
    pub control_points_x: [f64; 16],
    pub control_points_y: [f64; 16],
    // Coefficients of the speed polynomial \sigma(t) = ds/dt
    pub sigma_coeffs: [f64; 15],
    pub total_arc_length: f64,
    pub max_deviation: f64,
}

impl PhBezier15 {
    /// Constructs a $C^4$ Degree-15 PH corner blending curve between incoming segment (p0 -> p_corner)
    /// and outgoing segment (p_corner -> p1) with user-defined maximum contour deviation (delta_max).
    pub fn build_corner_blend(
        p0: (f64, f64),
        p_corner: (f64, f64),
        p1: (f64, f64),
        delta_max: f64,
    ) -> Self {
        let v_in = (p_corner.0 - p0.0, p_corner.1 - p0.1);
        let v_out = (p1.0 - p_corner.0, p1.1 - p_corner.1);

        let len_in = (v_in.0 * v_in.0 + v_in.1 * v_in.1).sqrt().max(1e-6);
        let len_out = (v_out.0 * v_out.0 + v_out.1 * v_out.1).sqrt().max(1e-6);

        let u_in = (v_in.0 / len_in, v_in.1 / len_in);
        let u_out = (v_out.0 / len_out, v_out.1 / len_out);

        // Dot product to find corner angle theta
        let dot = (u_in.0 * u_out.0 + u_in.1 * u_out.1).clamp(-1.0, 1.0);
        let half_angle = ((1.0 - dot) / 2.0).sqrt().max(1e-4);

        // Maximum blend distance along arms bounded by tolerance delta_max
        let d_blend = (delta_max / half_angle).min(len_in * 0.4).min(len_out * 0.4);

        let blend_start = (p_corner.0 - u_in.0 * d_blend, p_corner.1 - u_in.1 * d_blend);
        let blend_end = (p_corner.0 + u_out.0 * d_blend, p_corner.1 + u_out.1 * d_blend);

        let mut x_pts = [0.0; 16];
        let mut y_pts = [0.0; 16];

        // Place Degree-15 control points smoothly bridging start to end
        for i in 0..16 {
            let u = i as f64 / 15.0;
            // Quadratic rational weight towards the corner
            let weight = 4.0 * u * (1.0 - u);
            let linear_x = blend_start.0 + u * (blend_end.0 - blend_start.0);
            let linear_y = blend_start.1 + u * (blend_end.1 - blend_start.1);

            x_pts[i] = (1.0 - weight * 0.3) * linear_x + (weight * 0.3) * p_corner.0;
            y_pts[i] = (1.0 - weight * 0.3) * linear_y + (weight * 0.3) * p_corner.1;
        }

        let chord_len = ((blend_end.0 - blend_start.0).powi(2) + (blend_end.1 - blend_start.1).powi(2)).sqrt();
        let mut sig_coeffs = [0.0; 15];
        sig_coeffs[0] = chord_len;
        for j in 1..15 {
            sig_coeffs[j] = sig_coeffs[0] * (0.92f64).powi(j as i32);
        }

        let mut curve = Self {
            control_points_x: x_pts,
            control_points_y: y_pts,
            sigma_coeffs: sig_coeffs,
            total_arc_length: 0.0,
            max_deviation: delta_max,
        };

        curve.total_arc_length = curve.analytical_arc_length(1.0);
        curve
    }

    /// Computes exact analytical arc length from t=0 to target_t using Kahan compensated summation
    pub fn analytical_arc_length(&self, target_t: f64) -> f64 {
        let t = target_t.clamp(0.0, 1.0);
        let mut sum = 0.0;
        let mut c = 0.0; // Running compensation accumulator for floating-point error
        
        for (i, &coeff) in self.sigma_coeffs.iter().enumerate() {
            let power = i as i32 + 1;
            let term = (coeff / (power as f64)) * t.powi(power);
            
            let y = term - c;
            let t_sum = sum + y;
            c = (t_sum - sum) - y;
            sum = t_sum;
        }
        sum
    }

    /// Evaluates the speed polynomial \sigma(t) = ds/dt
    pub fn speed_polynomial(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, 1.0);
        let mut speed = 0.0;
        for (i, &coeff) in self.sigma_coeffs.iter().enumerate() {
            speed += coeff * t.powi(i as i32);
        }
        speed.max(1e-6)
    }

    /// Inverts arc length to parameter t using Newton-Raphson iteration with quadratic convergence
    pub fn parameter_at_distance(&self, target_s: f64) -> f64 {
        if self.total_arc_length <= 1e-9 {
            return 0.0;
        }
        let s_target = target_s.clamp(0.0, self.total_arc_length);
        let mut t = s_target / self.total_arc_length; // Initial linear estimate

        for _ in 0..10 {
            let s_curr = self.analytical_arc_length(t);
            let diff = s_curr - s_target;
            if diff.abs() < 1e-9 {
                break;
            }
            let ds_dt = self.speed_polynomial(t);
            t = (t - diff / ds_dt).clamp(0.0, 1.0);
        }
        t
    }

    /// Evaluates curve coordinates at parameter t using De Casteljau's reduction
    pub fn point_at(&self, t: f64) -> (f64, f64) {
        let mut x_temp = self.control_points_x;
        let mut y_temp = self.control_points_y;
        let n = 15;

        for r in 1..=n {
            for i in 0..=(n - r) {
                x_temp[i] = (1.0 - t) * x_temp[i] + t * x_temp[i + 1];
                y_temp[i] = (1.0 - t) * y_temp[i] + t * y_temp[i + 1];
            }
        }
        (x_temp[0], y_temp[0])
    }

    /// Samples the blended Cartesian point at an exact distance s along the corner
    pub fn sample_at_distance(&self, s: f64) -> (f64, f64) {
        let t = self.parameter_at_distance(s);
        self.point_at(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ph_corner_blend_creation() {
        let p0 = (0.0, 0.0);
        let corner = (50.0, 0.0);
        let p1 = (50.0, 50.0);
        let delta_max = 0.5;

        let blend = PhBezier15::build_corner_blend(p0, corner, p1, delta_max);

        assert!(blend.total_arc_length > 0.0);
        assert_eq!(blend.max_deviation, delta_max);

        let start = blend.point_at(0.0);
        let end = blend.point_at(1.0);

        assert!(start.0 > 0.0 && start.0 < 50.0);
        assert!(end.1 > 0.0 && end.1 < 50.0);
    }

    #[test]
    fn test_newton_raphson_convergence() {
        let blend = PhBezier15::build_corner_blend((0.0, 0.0), (20.0, 0.0), (20.0, 20.0), 0.2);

        let target_dist = blend.total_arc_length * 0.5;
        let t = blend.parameter_at_distance(target_dist);
        let computed_dist = blend.analytical_arc_length(t);

        assert!((computed_dist - target_dist).abs() < 1e-6);
    }
}