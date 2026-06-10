pub struct SafeHomingPlanner {
    pub max_accel: f64,
    pub safety_margin: f64,
}

impl SafeHomingPlanner {
    pub fn calculate_safe_velocity(&self, remaining_distance: f64, target_velocity: f64) -> f64 {
        if remaining_distance <= 0.0 || self.max_accel <= 0.0 || self.safety_margin <= 0.0 {
            return 0.0;
        }
        let safe_v = (2.0 * self.max_accel * remaining_distance * self.safety_margin).sqrt();
        if target_velocity < safe_v {
            target_velocity
        } else {
            safe_v
        }
    }

    pub fn evaluate_stopping_distance(&self, current_velocity: f64) -> f64 {
        if self.max_accel <= 0.0 {
            return f64::MAX;
        }
        (current_velocity * current_velocity) / (2.0 * self.max_accel)
    }
}