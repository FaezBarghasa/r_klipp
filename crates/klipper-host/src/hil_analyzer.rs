pub struct InputShaperCalibrator {
    pub sample_rate_hz: f64,
    pub scale_factor: f64,
}

impl InputShaperCalibrator {
    pub fn new(sample_rate_hz: f64, scale_factor: f64) -> Self {
        Self {
            sample_rate_hz,
            scale_factor,
        }
    }

    pub fn calculate_resonance_peaks(&self, samples: &[f32]) -> std::vec::Vec<f32> {
        let n_samples = samples.len();
        if n_samples == 0 {
            return std::vec::Vec::new();
        }

        let half_n = n_samples / 2;
        let mut magnitudes = std::vec::Vec::with_capacity(half_n);
        let pi = std::f64::consts::PI;

        for k in 0..half_n {
            let mut sum_cos: f64 = 0.0;
            let mut sum_sin: f64 = 0.0;

            for (n, &sample) in samples.iter().enumerate() {
                let w = 2.0 * pi * (k as f64) * (n as f64) / (n_samples as f64);
                sum_cos += (sample as f64) * w.cos();
                sum_sin += (sample as f64) * w.sin();
            }

            let magnitude = (sum_cos * sum_cos + sum_sin * sum_sin).sqrt();
            magnitudes.push((magnitude * self.scale_factor) as f32);
        }

        magnitudes
    }

    pub fn solve_zv_filter(&self, resonance_frequency_hz: f64, damping_ratio: f64) -> (f64, f64) {
        let pi = std::f64::consts::PI;
        let denominator = (1.0 - damping_ratio * damping_ratio).sqrt();
        let exponent = -(damping_ratio * pi) / denominator;
        let d = exponent.exp();
        
        let k_d = 1.0 / (1.0 + d);
        let delta_t_shaper = 1.0 / (2.0 * resonance_frequency_hz);

        (k_d, delta_t_shaper)
    }
}