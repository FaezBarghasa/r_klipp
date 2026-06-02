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
}
