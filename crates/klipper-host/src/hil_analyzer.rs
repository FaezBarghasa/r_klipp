// crates/klipper-host/src/hil_analyzer.rs

pub struct InputShaperCalibrator {
    pub sample_rate_hz: f64,
    pub scale_factor: f64,
}

impl InputShaperCalibrator {
    pub const fn new(sample_rate_hz: f64) -> Self {
        Self {
            sample_rate_hz,
            scale_factor: 1.0,
        }
    }

    /// Computes Discrete Fourier Transform magnitude array on input series.
    pub fn calculate_resonance_peaks(&self, samples: &[f32]) -> Vec<f32> {
        let n = samples.len();
        let mut dft_output = vec![0.0; n / 2];

        for k in 0..(n / 2) {
            let mut real = 0.0;
            let mut imag = 0.0;
            
            for (num, &val) in samples.iter().enumerate() {
                let angle = (2.0 * std::f64::consts::PI * (k * num) as f64) / n as f64;
                real += val as f64 * angle.cos();
                imag -= val as f64 * angle.sin();
            }
            dft_output[k] = (real * real + imag * imag).sqrt() as f32;
        }
        dft_output
    }

    /// Computes the exact physical parameters for a Zero-Vibration (ZV) filter.
    /// Returns the raw step delay (seconds) and damping ratio.
    pub fn solve_zv_filter(&self, resonance_frequency_hz: f64, damping_ratio: f64) -> (f64, f64) {
        let pi = std::f64::consts::PI;
        let d = (-damping_ratio * pi / (1.0 - damping_ratio * damping_ratio).sqrt()).exp();
        let scale_k = 1.0 / (1.0 + d);
        let time_delay = 1.0 / (2.0 * resonance_frequency_hz);
        
        (time_delay, scale_k)
    }

    /// Computes the exact physical parameters for a Minimum Zero-Vibration (MZV) filter.
    /// Returns the impulse amplitudes: (A1, A2, A3) and the damped period Td (seconds).
    pub fn solve_mzv_filter(&self, resonance_frequency_hz: f64, damping_ratio: f64) -> (f64, f64, f64, f64) {
        let pi = std::f64::consts::PI;
        let k = (-damping_ratio * pi / (1.0 - damping_ratio * damping_ratio).sqrt()).exp();
        
        let denom = 1.0 + 2.0 * k + k * k;
        let a1 = 1.0 / denom;
        let a2 = 2.0 * k / denom;
        let a3 = k * k / denom;
        
        let t_d = 1.0 / resonance_frequency_hz;
        (a1, a2, a3, t_d)
    }

    /// Identifies the dominant resonance frequency (Hz) in the calculated DFT peaks.
    pub fn identify_dominant_frequency(&self, peaks: &[f32]) -> f64 {
        if peaks.is_empty() {
            return 0.0;
        }
        let mut max_idx = 0;
        let mut max_val = peaks[0];
        for (i, &val) in peaks.iter().enumerate() {
            if val > max_val {
                max_val = val;
                max_idx = i;
            }
        }
        let n = (peaks.len() * 2) as f64;
        (max_idx as f64 * self.sample_rate_hz) / n
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vibration_analyzer_and_shaper() {
        let analyzer = InputShaperCalibrator::new(3200.0); // 3.2 kHz sample rate
        
        // Generate simulated acceleration data with a 50 Hz sine wave
        let n = 256;
        let mut samples = vec![0.0f32; n];
        for i in 0..n {
            let t = i as f64 / 3200.0;
            samples[i] = (2.0 * std::f64::consts::PI * 50.0 * t).sin() as f32;
        }

        let peaks = analyzer.calculate_resonance_peaks(&samples);
        let dominant_f = analyzer.identify_dominant_frequency(&peaks);
        
        // Verify we detect dominant frequency close to 50 Hz
        assert!((dominant_f - 50.0).abs() < 15.0);

        // Solve ZV and MZV filters
        let (zv_delay, zv_scale) = analyzer.solve_zv_filter(dominant_f, 0.1);
        assert!(zv_delay > 0.0);
        assert!(zv_scale > 0.0);

        let (a1, a2, a3, mzv_td) = analyzer.solve_mzv_filter(dominant_f, 0.1);
        assert!((a1 + a2 + a3 - 1.0).abs() < 1e-5);
        assert!(mzv_td > 0.0);
    }
}
