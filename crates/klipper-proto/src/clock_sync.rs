//! Clock synchronization using least-squares linear regression.
//!
//! Maps host system times (microseconds) to MCU timer ticks.

use heapless::Vec;

#[derive(Debug, Clone, Copy, Default)]
pub struct ClockSample {
    pub host_us: u64,
    pub mcu_ticks: u64,
}

pub struct ClockSynchronizer {
    samples: Vec<ClockSample, 32>,
    alpha: f64,
    beta: f64,
}

impl ClockSynchronizer {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            alpha: 1.0,
            beta: 0.0,
        }
    }

    pub fn add_sample(&mut self, host_us: u64, mcu_ticks: u64) {
        if self.samples.is_full() {
            // Remove oldest sample
            self.samples.remove(0);
        }
        let _ = self.samples.push(ClockSample { host_us, mcu_ticks });
        self.recalculate();
    }

    fn recalculate(&mut self) {
        let n = self.samples.len() as f64;
        if n < 2.0 {
            return;
        }

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xy = 0.0;
        let mut sum_xx = 0.0;

        for sample in &self.samples {
            let x = sample.host_us as f64;
            let y = sample.mcu_ticks as f64;
            sum_x += x;
            sum_y += y;
            sum_xy += x * y;
            sum_xx += x * x;
        }

        let denominator = n * sum_xx - sum_x * sum_x;
        if denominator.abs() < 1e-9 {
            return;
        }

        self.alpha = (n * sum_xy - sum_x * sum_y) / denominator;
        self.beta = (sum_y - self.alpha * sum_x) / n;
    }

    pub fn translate_host_to_mcu_ticks(&self, host_us: u64) -> u32 {
        let mcu_ticks_f = self.alpha * (host_us as f64) + self.beta;
        mcu_ticks_f.round() as u32
    }
}
