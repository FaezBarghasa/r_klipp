use core::sync::atomic::{AtomicBool, Ordering};

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

/// Double-buffered atomic wrapper for thread-safe lock-free updates of ClockSyncModel.
pub struct SharedClockModel {
    active: AtomicBool,
    models: [ClockSyncModel; 2],
}

impl SharedClockModel {
    pub const fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            models: [ClockSyncModel::new(), ClockSyncModel::new()],
        }
    }

    /// Read the active model (lock-free, safe to call from Priority 4 ISR)
    #[inline(always)]
    pub fn get_active(&self) -> ClockSyncModel {
        let idx = self.active.load(Ordering::Acquire) as usize;
        self.models[idx]
    }

    /// Update the inactive model and swap (called from Priority 1 task)
    pub fn update(&mut self, local_history: &[u32], master_history: &[u64]) {
        let active_idx = self.active.load(Ordering::Relaxed);
        let inactive_idx = !active_idx;
        
        let mut model = self.models[active_idx as usize];
        model.update_regression_model(local_history, master_history);
        
        self.models[inactive_idx as usize] = model;
        self.active.store(inactive_idx, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shared_clock_model_sync() {
        let mut shared = SharedClockModel::new();
        
        // Assert initial slope/intercept
        let active = shared.get_active();
        assert_eq!(active.slope, 1.0);
        
        // Update regression
        let local_history = [1000, 2000, 3000];
        let master_history = [1005, 2010, 3015];
        shared.update(&local_history, &master_history);

        // Verify updated active model parameters
        let active_updated = shared.get_active();
        assert!((active_updated.slope - 1.01).abs() < 1e-4);
        assert!((active_updated.intercept - (-5.0)).abs() < 1e-4);
    }
}
