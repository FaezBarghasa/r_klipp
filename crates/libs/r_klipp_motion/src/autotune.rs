
// Goertzel algorithm for single-tone detection
pub fn goertzel(samples: &[f32], freq: f32, sample_rate: f32) -> f32 {
    let k = (0.5 + samples.len() as f32 * freq / sample_rate) as usize;
    let omega = (2.0 * core::f32::consts::PI * k as f32) / samples.len() as f32;
    let cosine = omega.cos();
    let sine = omega.sin();
    let coeff = 2.0 * cosine;

    let mut q0 = 0.0;
    let mut q1 = 0.0;
    let mut q2 = 0.0;

    for sample in samples {
        q0 = coeff * q1 - q2 + sample;
        q2 = q1;
        q1 = q0;
    }

    (q1.powi(2) + q2.powi(2) - q1 * q2 * coeff).sqrt()
}

// Zero Vibration Derivative (ZVD) Input Shaper
pub struct ZvdShaper {
    coeffs: [f32; 2],
    times: [f32; 2],
}

impl ZvdShaper {
    pub fn new(resonant_freq: f32, damping_ratio: f32) -> Self {
        let k = (-(damping_ratio * core::f32::consts::PI) / (1.0 - damping_ratio.powi(2)).sqrt()).exp();
        let t_d = 1.0 / (resonant_freq * (1.0 - damping_ratio.powi(2)).sqrt());

        Self {
            coeffs: [1.0 / (1.0 + k), k / (1.0 + k)],
            times: [0.0, t_d / 2.0],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goertzel_algorithm() {
        // Create a sine wave at 50Hz
        let sample_rate = 1000.0;
        let freq = 50.0;
        let mut samples = [0.0; 100];
        for i in 0..100 {
            samples[i] = (2.0 * core::f32::consts::PI * freq * (i as f32 / sample_rate)).sin();
        }

        let power_at_50hz = goertzel(&samples, 50.0, sample_rate);
        let power_at_100hz = goertzel(&samples, 100.0, sample_rate);

        assert!(power_at_50hz > power_at_100hz * 10.0);
    }
}
