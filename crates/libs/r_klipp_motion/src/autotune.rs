use micromath::F32Ext;

pub struct Goertzel {
    s1: f32,
    s2: f32,
    coeff: f32,
}

impl Goertzel {
    pub fn new(freq: f32, sample_rate: f32) -> Self {
        let k = (0.5 + (1024.0 * freq / sample_rate)) as u32;
        let omega = (2.0 * 3.14159 * k as f32) / 1024.0;
        Self {
            s1: 0.0,
            s2: 0.0,
            coeff: 2.0 * omega.cos(),
        }
    }

    pub fn process_sample(&mut self, sample: f32) {
        let s0 = sample + self.coeff * self.s1 - self.s2;
        self.s2 = self.s1;
        self.s1 = s0;
    }

    pub fn get_magnitude_squared(&self) -> f32 {
        self.s1 * self.s1 + self.s2 * self.s2 - self.coeff * self.s1 * self.s2
    }
}

pub fn calculate_zvd_shaper(resonant_freq: f32) -> (f32, f32) {
    let t_res = 1.0 / resonant_freq;
    let k = (-3.14159).exp();
    let d = 1.0 / (1.0 + k);
    (d, t_res)
}
