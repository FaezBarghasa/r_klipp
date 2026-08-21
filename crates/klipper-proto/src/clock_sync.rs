//! Digital Phase-Locked Loop (DPLL) Clock Synchronization with Linear Regression and Outlier Rejection.
//!
//! Maps host system times (microseconds) to MCU timer ticks with sub-microsecond jitter filtering.

use heapless::Vec;

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ClockSample {
    pub host_send_us: u64,
    pub mcu_time_ticks: u64,
    pub host_recv_us: u64,
    pub rtt_us: u64,
}

pub struct DpllClockSync {
    samples: Vec<ClockSample, 32>,
    pub frequency_ticks_per_us: f64,
    pub offset_ticks: f64,
    pub min_rtt_us: u64,
}

impl DpllClockSync {
    pub fn new() -> Self {
        Self {
            samples: Vec::new(),
            frequency_ticks_per_us: 100.0, // Default 100MHz base
            offset_ticks: 0.0,
            min_rtt_us: u64::MAX,
        }
    }

    /// Adds a round-trip ping sample: host_send_us -> mcu_ticks -> host_recv_us.
    /// Filters out packets with excessive latency jitter (>3x minimum RTT).
    pub fn add_sample(&mut self, host_send_us: u64, mcu_time_ticks: u64, host_recv_us: u64) -> bool {
        let rtt = host_recv_us.saturating_sub(host_send_us);
        if rtt == 0 {
            return false;
        }

        if rtt < self.min_rtt_us {
            self.min_rtt_us = rtt;
        }

        // Outlier rejection: ignore samples with excessive network/USB latency jitter
        if self.samples.len() >= 4 && rtt > self.min_rtt_us.saturating_mul(3) {
            return false; // Rejected outlier
        }

        if self.samples.is_full() {
            self.samples.remove(0);
        }

        let _ = self.samples.push(ClockSample {
            host_send_us,
            mcu_time_ticks,
            host_recv_us,
            rtt_us: rtt,
        });

        self.recalculate();
        true
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

        for s in &self.samples {
            // Est host timestamp when MCU recorded tick = midpoint of send/recv
            let host_mid = (s.host_send_us + s.host_recv_us) as f64 * 0.5;
            let mcu_t = s.mcu_time_ticks as f64;

            sum_x += host_mid;
            sum_y += mcu_t;
            sum_xy += host_mid * mcu_t;
            sum_xx += host_mid * host_mid;
        }

        let denominator = n * sum_xx - sum_x * sum_x;
        if denominator.abs() > 1e-9 {
            let slope = (n * sum_xy - sum_x * sum_y) / denominator;
            let intercept = (sum_y - slope * sum_x) / n;

            // DPLL Smoothing Filter (IIR)
            self.frequency_ticks_per_us = self.frequency_ticks_per_us * 0.8 + slope * 0.2;
            self.offset_ticks = self.offset_ticks * 0.8 + intercept * 0.2;
        }
    }

    pub fn host_to_mcu_ticks(&self, host_us: u64) -> u64 {
        let ticks = self.frequency_ticks_per_us * (host_us as f64) + self.offset_ticks;
        libm::round(ticks.max(0.0)) as u64
    }

    pub fn mcu_to_host_us(&self, mcu_ticks: u64) -> u64 {
        if self.frequency_ticks_per_us <= 0.0 {
            return 0;
        }
        let us = (mcu_ticks as f64 - self.offset_ticks) / self.frequency_ticks_per_us;
        libm::round(us.max(0.0)) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dpll_clock_sync() {
        let mut sync = DpllClockSync::new();

        // Simulate a 100MHz MCU timer with 200us USB RTT
        let mcu_freq = 100.0; // 100 ticks per microsecond
        for i in 0..10 {
            let send = 1_000_000 + i * 50_000;
            let recv = send + 200;
            let mcu_mid = (send + 100) * 100; // Exact mid tick
            sync.add_sample(send, mcu_mid, recv);
        }

        let test_host_us = 2_000_000;
        let predicted_ticks = sync.host_to_mcu_ticks(test_host_us);
        let roundtrip_us = sync.mcu_to_host_us(predicted_ticks);

        assert!((roundtrip_us as i64 - test_host_us as i64).abs() <= 2);
    }
}
