// crates/klipper-mcu-firmware/src/clock_sync.rs

/// Real-Time Clock alignment parameters matching y = mx + c.
#[derive(Copy, Clone, Debug)]
pub struct ClockSyncModel {
    /// Linear clock speed scale factor.
    pub slope: f64,
    /// Absolute microsecond offset baseline.
    pub intercept: f64,
    /// Snapshot of local ticks at last model update.
    pub last_local_ticks: u32,
}

impl ClockSyncModel {
    pub const fn new() -> Self {
        Self {
            slope: 1.0,
            intercept: 0.0,
            last_local_ticks: 0,
        }
    }

    /// Computes synchronized global master timestamp from raw local hardware ticks.
    #[inline(always)]
    pub fn local_to_master(&self, local_ticks: u32) -> u64 {
        let delta = local_ticks.wrapping_sub(self.last_local_ticks) as f64;
        let master_base = (self.last_local_ticks as f64) * self.slope + self.intercept;
        (master_base + delta * self.slope) as u64
    }

    /// Updates clock parameters using linear regression mapping (recursive least squares).
    pub fn update_regression_model(&mut self, local_history: &[u32], master_history: &[u64]) {
        let n = local_history.len() as f64;
        if n < 2.0 { return; }

        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xx = 0.0;
        let mut sum_xy = 0.0;

        for i in 0..local_history.len() {
            let x = local_history[i] as f64;
            let y = master_history[i] as f64;
            sum_x += x;
            sum_y += y;
            sum_xx += x * x;
            sum_xy += x * y;
        }

        let denominator = n * sum_xx - sum_x * sum_x;
        if denominator.abs() > 1e-9 {
            self.slope = (n * sum_xy - sum_x * sum_y) / denominator;
            self.intercept = (sum_y - self.slope * sum_x) / n;
            self.last_local_ticks = local_history[local_history.len() - 1];
        }
    }
}
