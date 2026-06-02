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
        // v_max = sqrt(2 * a * d * margin)
        let limit_velocity = (2.0 * self.max_accel * remaining_distance * self.safety_margin).sqrt();
        
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

    /// Verifies if a move is safe to execute given the remaining distance and allowable overshoot limit.
    pub fn is_move_safe(&self, current_velocity: f64, remaining_distance: f64, overshoot_margin: f64) -> bool {
        let stop_dist = self.evaluate_stopping_distance(current_velocity);
        stop_dist <= remaining_distance + overshoot_margin
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_homing_velocity_scaling() {
        let planner = SafeHomingPlanner::new(1000.0); // max_accel = 1000 mm/s^2

        // Remaining distance is large, target velocity should be preserved
        let safe_vel_large = planner.calculate_safe_velocity(100.0, 50.0);
        assert_eq!(safe_vel_large, 50.0);

        // Remaining distance is extremely small, target velocity should be scaled down
        let safe_vel_small = planner.calculate_safe_velocity(0.5, 50.0);
        assert!(safe_vel_small < 50.0);
    }

    #[test]
    fn test_homing_safety_checks() {
        let planner = SafeHomingPlanner::new(1000.0);

        // Stopping distance for 40 mm/s is 40^2 / 2000 = 0.8 mm
        let stopping_dist = planner.evaluate_stopping_distance(40.0);
        assert!((stopping_dist - 0.8).abs() < 1e-6);

        // Safe if remaining distance + overshoot is 1.0 mm
        assert!(planner.is_move_safe(40.0, 0.5, 0.5)); // 0.8 <= 1.0 -> true

        // Unsafe if remaining distance + overshoot is 0.5 mm
        assert!(!planner.is_move_safe(40.0, 0.2, 0.2)); // 0.8 <= 0.4 -> false
    }
}
