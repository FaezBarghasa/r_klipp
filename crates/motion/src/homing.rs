use crate::g4_planner::KinematicLimits;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomingError {
    OvershootExceeded,
    InvalidAcceleration,
    InvalidVelocity,
    VelocityExceedsLimit,
}

impl core::fmt::Display for HomingError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::OvershootExceeded => write!(f, "Homing velocity exceeds overshoot safety limits. Deceleration distance too large."),
            Self::InvalidAcceleration => write!(f, "Kinematic limits specify zero or negative max acceleration, which is invalid."),
            Self::InvalidVelocity => write!(f, "Homing velocity is NaN or infinite, violating deterministic bounds."),
            Self::VelocityExceedsLimit => write!(f, "Homing velocity exceeds the configured maximum kinematic velocity."),
        }
    }
}



pub struct SafeHomingPlanner {
    /// Kinematic boundaries for homing motion
    pub limits: KinematicLimits,
    pub max_overshoot_distance: f64,
    /// Pre-calculated maximum velocity allowed by the combined overshoot and jerk/snap constraints.
    pub cached_safe_limit: f64,
}

impl SafeHomingPlanner {
    pub fn new(limits: KinematicLimits, max_overshoot_distance: f64) -> Self {
        let overshoot = max_overshoot_distance.max(0.0);
        
        // Analytical stopping distance accounting for deceleration ramp-up time:
        // t_ramp = a_max / j_max
        // d_decel = (v^2) / (2 * a_max) + v * (a_max / (2 * j_max))
        // Solving for v gives the robust physical speed ceiling.
        let safe_limit = if limits.max_accel > 0.0 {
            let a = limits.max_accel;
            let j = limits.max_jerk.max(1.0);
            let ramp_time = a / j;
            
            // Quadratic root for v: v^2 / (2a) + v * (ramp_time/2) - overshoot = 0
            // v = (-ramp_time/2 + sqrt((ramp_time/2)^2 + 4 * (1/2a) * overshoot)) / (1/a)
            let b = ramp_time * 0.5;
            let disc = b * b + (2.0 * overshoot / a);
            if disc >= 0.0 {
                let v = (-b + disc.sqrt()) * a;
                v.max(0.0)
            } else {
                (2.0 * a * overshoot).sqrt()
            }
        } else {
            0.0
        };

        Self {
            limits,
            max_overshoot_distance: overshoot,
            cached_safe_limit: safe_limit.min(limits.max_velocity),
        }
    }

    /// Validates the trajectory before launching the raw step-pulse sequence.
    pub fn validate_homing_move(&self, velocity: f64) -> Result<(), HomingError> {
        if !velocity.is_finite() {
            return Err(HomingError::InvalidVelocity);
        }

        if velocity.abs() > self.limits.max_velocity {
            return Err(HomingError::VelocityExceedsLimit);
        }

        if self.limits.max_accel <= 0.0 {
            return Err(HomingError::InvalidAcceleration);
        }

        if velocity.abs() > self.cached_safe_limit + 1e-6 {
            return Err(HomingError::OvershootExceeded);
        }
        Ok(())
    }

    /// Computes the maximum safe homing velocity that guarantees the carriage
    /// can halt within `max_overshoot_distance` if the endstop fails or stalls.
    pub fn calculate_safe_velocity(&self, target_velocity: f64) -> f64 {
        if self.limits.max_accel <= 0.0 || target_velocity.is_nan() {
            return 0.0;
        }

        if target_velocity.is_infinite() {
            return if target_velocity.is_sign_positive() {
                self.cached_safe_limit
            } else {
                -self.cached_safe_limit
            };
        }

        target_velocity.clamp(-self.cached_safe_limit, self.cached_safe_limit)
    }

    /// Returns the maximum allowed deceleration stopping distance for a given homing velocity
    pub fn stopping_distance_at(&self, velocity: f64) -> f64 {
        if self.limits.max_accel <= 0.0 {
            return f64::INFINITY;
        }
        let v = velocity.abs();
        let a = self.limits.max_accel;
        let j = self.limits.max_jerk.max(1.0);
        (v * v) / (2.0 * a) + v * (a / (2.0 * j))
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    fn mock_limits(v: f64, a: f64) -> KinematicLimits {
        KinematicLimits {
            max_velocity: v,
            max_accel: a,
            max_jerk: 50000.0,
            max_snap: 500000.0,
            max_crackle: 5000000.0,
        }
    }

    #[test]
    fn test_homing_error_variants() {
        assert_eq!(HomingError::OvershootExceeded, HomingError::OvershootExceeded);
        assert_eq!(HomingError::InvalidAcceleration, HomingError::InvalidAcceleration);
        assert_eq!(HomingError::InvalidVelocity, HomingError::InvalidVelocity);
        assert_eq!(HomingError::VelocityExceedsLimit, HomingError::VelocityExceedsLimit);
    }

    #[test]
    fn test_planner_initialization() {
        // v_safe calculated via jerk-bounded quadratic decel root
        let limits = mock_limits(50.0, 100.0);
        let planner = SafeHomingPlanner::new(limits, 2.0);
        assert!((planner.cached_safe_limit - 19.90025).abs() < 1e-4);

        // Check that it respects max_velocity if sqrt(2ad) is higher
        let limits_slow = mock_limits(10.0, 100.0);
        let planner_slow = SafeHomingPlanner::new(limits_slow, 2.0);
        assert_eq!(planner_slow.cached_safe_limit, 10.0);
    }

    #[test]
    fn test_validate_homing_move() {
        let limits = mock_limits(20.0, 50.0);
        let planner = SafeHomingPlanner::new(limits, 1.0);

        // Valid cases within jerk-limited boundary
        assert!(planner.validate_homing_move(5.0).is_ok());
        assert!(planner.validate_homing_move(-5.0).is_ok());
        assert!(planner.validate_homing_move(9.97).is_ok());

        // Exceeds overshoot safety
        assert_eq!(
            planner.validate_homing_move(11.0),
            Err(HomingError::OvershootExceeded)
        );

        // Exceeds machine kinematic limit (even if overshoot allows it)
        let planner_high_overshoot = SafeHomingPlanner::new(mock_limits(20.0, 50.0), 100.0);
        assert_eq!(
            planner_high_overshoot.validate_homing_move(25.0),
            Err(HomingError::VelocityExceedsLimit)
        );

        // Invalid numbers
        assert_eq!(
            planner.validate_homing_move(f64::NAN),
            Err(HomingError::InvalidVelocity)
        );
    }

    #[test]
    fn test_calculate_safe_velocity() {
        let limits = mock_limits(20.0, 50.0);
        let planner = SafeHomingPlanner::new(limits, 1.0);
        let expected_max = planner.cached_safe_limit;

        // Clamping logic
        assert!((planner.calculate_safe_velocity(15.0) - expected_max).abs() < 1e-6);
        assert!((planner.calculate_safe_velocity(-15.0) + expected_max).abs() < 1e-6);
        assert_eq!(planner.calculate_safe_velocity(5.0), 5.0);

        // Edge cases
        assert_eq!(planner.calculate_safe_velocity(f64::NAN), 0.0);
        assert!((planner.calculate_safe_velocity(f64::INFINITY) - expected_max).abs() < 1e-6);
    }

    #[test]
    fn test_invalid_kinematics_handling() {
        // Zero acceleration case
        let limits = mock_limits(20.0, 0.0);
        let planner = SafeHomingPlanner::new(limits, 1.0);
        
        assert_eq!(
            planner.validate_homing_move(5.0),
            Err(HomingError::InvalidAcceleration)
        );
        assert_eq!(planner.calculate_safe_velocity(5.0), 0.0);

        // Negative overshoot treated as zero
        let planner_neg = SafeHomingPlanner::new(mock_limits(20.0, 50.0), -5.0);
        assert_eq!(planner_neg.max_overshoot_distance, 0.0);
        assert_eq!(planner_neg.cached_safe_limit, 0.0);
    }
}