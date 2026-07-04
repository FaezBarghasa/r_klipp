use micromath::F32Ext;

pub struct Goertzel {
    q1: f32,
    q2: f32,
    coeff: f32,
}

impl Goertzel {
    pub fn new(target_freq: f32, sample_rate: f32) -> Self {
        let k = (0.5 + (target_freq / sample_rate) * (sample_rate as u32) as f32) as u32;
        let omega = (2.0 * core::f32::consts::PI * k as f32) / sample_rate;
        Self {
            q1: 0.0,
            q2: 0.0,
            coeff: 2.0 * omega.cos(),
        }
    }

    pub fn process_sample(&mut self, sample: f32) {
        let q0 = self.coeff * self.q1 - self.q2 + sample;
        self.q2 = self.q1;
        self.q1 = q0;
    }

    pub fn get_magnitude_squared(&self) -> f32 {
        self.q1 * self.q1 + self.q2 * self.q2 - self.q1 * self.q2 * self.coeff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_goertzel_detects_frequency() {
        let sample_rate = 1000.0;
        let target_freq = 45.0;
        let mut goertzel = Goertzel::new(target_freq, sample_rate);
        let mut noise_goertzel = Goertzel::new(100.0, sample_rate); // a different frequency

        for i in 0..1000 {
            let time = i as f32 / sample_rate;
            let signal = (2.0 * core::f32::consts::PI * target_freq * time).sin();
            goertzel.process_sample(signal);
            noise_goertzel.process_sample(signal);
        }

        let mag_at_target = goertzel.get_magnitude_squared();
        let mag_at_noise = noise_goertzel.get_magnitude_squared();

        assert!(mag_at_target > 1000.0, "Magnitude at target freq was {}", mag_at_target);
        assert!(mag_at_noise < 1.0, "Magnitude at noise freq was {}", mag_at_noise);
    }
}
