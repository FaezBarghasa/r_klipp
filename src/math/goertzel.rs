// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use libm::{cos, sin, sqrt};

pub struct Goertzel {
    s_prev: f32,
    s_prev2: f32,
    coeff: f32,
}

impl Goertzel {
    pub fn new(target_freq: f32, sample_rate: f32) -> Self {
        let k = (0.5 + (target_freq / sample_rate) * 1024.0) as u32; // Assuming N=1024 for block size
        let omega = (2.0 * core::f32::consts::PI * k as f32) / 1024.0;
        Self {
            s_prev: 0.0,
            s_prev2: 0.0,
            coeff: 2.0 * cos(omega) as f32,
        }
    }

    pub fn process_sample(&mut self, sample: f32) {
        let s = sample + self.coeff * self.s_prev - self.s_prev2;
        self.s_prev2 = self.s_prev;
        self.s_prev = s;
    }

    pub fn get_magnitude_squared(&self) -> f32 {
        self.s_prev * self.s_prev + self.s_prev2 * self.s_prev2
            - self.coeff * self.s_prev * self.s_prev2
    }

    pub fn get_magnitude(&self) -> f32 {
        sqrt(self.get_magnitude_squared())
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

        for i in 0..1024 {
            let time = i as f32 / sample_rate;
            let sample = sin(2.0 * core::f32::consts::PI * target_freq * time);
            goertzel.process_sample(sample);
        }

        let magnitude = goertzel.get_magnitude();
        // The magnitude should be high for the target frequency.
        // The exact value depends on N and windowing, but it should be significant.
        assert!(magnitude > 100.0);
    }

    #[test]
    fn test_goertzel_rejects_other_frequency() {
        let sample_rate = 1000.0;
        let target_freq = 45.0;
        let other_freq = 100.0;
        let mut goertzel = Goertzel::new(target_freq, sample_rate);

        for i in 0..1024 {
            let time = i as f32 / sample_rate;
            let sample = sin(2.0 * core::f32::consts::PI * other_freq * time);
            goertzel.process_sample(sample);
        }

        let magnitude = goertzel.get_magnitude();
        assert!(magnitude < 1.0);
    }
}