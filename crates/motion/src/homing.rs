// crates/motion/src/homing.rs

pub struct SafeHomingPlanner {
    pub max_accel: f64,
    pub safety_margin: f64,
}

impl SafeHomingPlanner {
    pub const fn new(max_accel: f64) -> Self {
        Self {
            max_accel,
            safety_margin: 0.85,
        }
    }

    /// Evaluates maximum safe homing speed given distance to physical hardstop.
    /// Returns a scaled, safe velocity (mm/s).
    pub fn calculate_safe_velocity(&self, remaining_distance: f64, target_velocity: f64) -> f64 {
        // v_max = sqrt(2 * a * d) * margin
        let limit_velocity = (2.0 * self.max_accel * remaining_distance).sqrt() * self.safety_margin;
        
        if target_velocity > limit_velocity {
            limit_velocity
        } else {
            target_velocity
        }
    }

    /// Checks if a sudden, unexpected endstop trigger can be stopped within structural limits.
    pub fn evaluate_stopping_distance(&self, current_velocity: f64) -> f64 {
        (current_velocity * current_velocity) / (2.0 * self.max_accel)
    }
}
