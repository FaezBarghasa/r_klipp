#[derive(Debug, Clone, Copy)]
pub struct ClockSyncModel {
    pub slope: f64,
    pub intercept: f64,
    pub last_local_ticks: u32,
    pub is_locked: bool,
}

impl ClockSyncModel {
    pub fn new() -> Self {
        Self {
            slope: 1.0,
            intercept: 0.0,
            last_local_ticks: 0,
            is_locked: false,
        }
    }

    pub fn local_to_master(&self, local_ticks: u32) -> u64 {
        let delta_t = local_ticks.wrapping_sub(self.last_local_ticks) as f64;
        let t_last_local = self.last_local_ticks as f64;
        let t_master = (t_last_local * self.slope) + self.intercept + (delta_t * self.slope);
        t_master as u64
    }

    pub fn update_synchronization(&mut self, local_history: &[u32], master_history: &[u64]) {
        if local_history.is_empty() || local_history.len() != master_history.len() {
            return;
        }

        let n = local_history.len() as f64;
        let x0 = local_history[0];
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_xx = 0.0;
        let mut sum_xy = 0.0;

        for i in 0..local_history.len() {
            let x = local_history[i].wrapping_sub(x0) as f64;
            let y = master_history[i] as f64;
            
            sum_x += x;
            sum_y += y;
            sum_xx += x * x;
            sum_xy += x * y;
        }

        let denominator = n * sum_xx - sum_x * sum_x;
        let abs_denom = if denominator < 0.0 { -denominator } else { denominator };
        
        if abs_denom > 1e-9 {
            self.slope = (n * sum_xy - sum_x * sum_y) / denominator;
            let local_intercept = (sum_y - self.slope * sum_x) / n;
            
            self.intercept = local_intercept - self.slope * (x0 as f64);
            self.last_local_ticks = *local_history.last().unwrap();
            self.is_locked = true;
        }
    }
}