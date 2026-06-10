#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlannerError {
    ConvergenceFailed,
}

pub struct PhBezier15 {
    pub cx: [f64; 16],
    pub cy: [f64; 16],
    pub sigma_coeffs: [f64; 15],
}

impl PhBezier15 {
    pub fn evaluate_arc_length(&self, t: f64) -> f64 {
        let mut sum = 0.0;
        let mut c = 0.0;
        let mut t_pow = t;
        for i in 0..15 {
            let term = self.sigma_coeffs[i] * t_pow;
            let y = term - c;
            let t_sum = sum + y;
            c = (t_sum - sum) - y;
            sum = t_sum;
            t_pow *= t;
        }
        sum
    }

    pub fn evaluate_sigma(&self, t: f64) -> f64 {
        let mut sum = 0.0;
        let mut c = 0.0;
        let mut t_pow = 1.0;
        for i in 0..15 {
            let j = (i + 1) as f64;
            let term = self.sigma_coeffs[i] * j * t_pow;
            let y = term - c;
            let t_sum = sum + y;
            c = (t_sum - sum) - y;
            sum = t_sum;
            t_pow *= t;
        }
        sum
    }

    pub fn get_t_from_distance(&self, s: f64) -> Result<f64, PlannerError> {
        let mut t_n = 0.5;
        for _ in 0..8 {
            let s_n = self.evaluate_arc_length(t_n);
            let diff = s_n - s;
            if diff.abs() < 1e-9 {
                return Ok(t_n);
            }
            let sigma = self.evaluate_sigma(t_n);
            t_n -= diff / sigma;
        }
        
        let final_s = self.evaluate_arc_length(t_n);
        if (final_s - s).abs() < 1e-9 {
            Ok(t_n)
        } else {
            Err(PlannerError::ConvergenceFailed)
        }
    }

    pub fn evaluate_position(&self, t: f64) -> (f64, f64) {
        let mut px = self.cx;
        let mut py = self.cy;
        let u = 1.0 - t;
        for i in 1..16 {
            for j in 0..(16 - i) {
                px[j] = u * px[j] + t * px[j + 1];
                py[j] = u * py[j] + t * py[j + 1];
            }
        }
        (px[0], py[0])
    }
}