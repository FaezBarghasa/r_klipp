use crate::g4_planner::KinematicLimits;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HomingError {
    OvershootExceeded,
    InvalidAcceleration,
    InvalidVelocity,
    VelocityExceedsLimit,
}

impl std::fmt::Display for HomingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OvershootExceeded => write!(f, "Homing velocity exceeds overshoot safety limits. Deceleration distance too large."),
            Self::InvalidAcceleration => write!(f, "Kinematic limits specify zero or negative max acceleration, which is invalid."),
            Self::InvalidVelocity => write!(f, "Homing velocity is NaN or infinite, violating deterministic bounds."),
            Self::VelocityExceedsLimit => write!(f, "Homing velocity exceeds the configured maximum kinematic velocity."),
        }
    }
}

impl std::error::Error for HomingError {}

pub struct SafeHomingPlanner {
    limits: KinematicLimits,
    max_overshoot_distance: f64,
    /// Pre-calculated maximum velocity allowed by the overshoot constraint.
    /// This avoids repeated `sqrt` and division during runtime.
    cached_safe_limit: f64,
}

impl SafeHomingPlanner {
    pub fn new(limits: KinematicLimits, max_overshoot_distance: f64) -> Self {
        let overshoot = max_overshoot_distance.max(0.0);
        
        // Calculate the physical limit imposed by the overshoot distance:
        // v = sqrt(2 * a * d)
        let safe_limit = if limits.max_accel > 0.0 {
            (2.0 * limits.max_accel * overshoot).sqrt()
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

        // Using pre-calculated cached_safe_limit is more performant than
        // calculating decel_distance = v^2 / 2a every time.
        // We use a small epsilon or direct comparison because cached_safe_limit 
        // already accounts for max_velocity and max_accel.
        if velocity.abs() > self.cached_safe_limit + f64::EPSILON {
            return Err(HomingError::OvershootExceeded);
        }
        Ok(())
    }

    /// Computes the maximum safe homing velocity that guarantees the carriage
    /// can halt within `max_overshoot_distance` if the endstop fails.
    pub fn calculate_safe_velocity(&self, target_velocity: f64) -> f64 {
        if !target_velocity.is_finite() || self.limits.max_accel <= 0.0 {
            return 0.0;
        }

        target_velocity.clamp(-self.cached_safe_limit, self.cached_safe_limit)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mock_limits(v: f64, a: f64) -> KinematicLimits {
        KinematicLimits {
            max_velocity: v,
            max_accel: a,
        }
    }

    #[test]
    fn test_homing_error_display() {
        assert!(format!("{}", HomingError::OvershootExceeded).contains("overshoot"));
        assert!(format!("{}", HomingError::InvalidAcceleration).contains("acceleration"));
        assert!(format!("{}", HomingError::InvalidVelocity).contains("NaN"));
        assert!(format!("{}", HomingError::VelocityExceedsLimit).contains("maximum"));
    }

    #[test]
    fn test_planner_initialization() {
        // v = sqrt(2 * 100 * 2) = sqrt(400) = 20.0
        let limits = mock_limits(50.0, 100.0);
        let planner = SafeHomingPlanner::new(limits, 2.0);
        assert_eq!(planner.cached_safe_limit, 20.0);

        // Check that it respects max_velocity if sqrt(2ad) is higher
        let limits_slow = mock_limits(10.0, 100.0);
        let planner_slow = SafeHomingPlanner::new(limits_slow, 2.0);
        assert_eq!(planner_slow.cached_safe_limit, 10.0);
    }

    #[test]
    fn test_validate_homing_move() {
        // v_safe = sqrt(2 * 50 * 1) = 10.0
        let limits = mock_limits(20.0, 50.0);
        let planner = SafeHomingPlanner::new(limits, 1.0);

        // Valid cases
        assert!(planner.validate_homing_move(5.0).is_ok());
        assert!(planner.validate_homing_move(-5.0).is_ok());
        assert!(planner.validate_homing_move(10.0).is_ok());

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
        // v_safe = 10.0
        let limits = mock_limits(20.0, 50.0);
        let planner = SafeHomingPlanner::new(limits, 1.0);

        // Clamping logic
        assert_eq!(planner.calculate_safe_velocity(15.0), 10.0);
        assert_eq!(planner.calculate_safe_velocity(-15.0), -10.0);
        assert_eq!(planner.calculate_safe_velocity(5.0), 5.0);

        // Edge cases
        assert_eq!(planner.calculate_safe_velocity(f64::NAN), 0.0);
        assert_eq!(planner.calculate_safe_velocity(f64::INFINITY), 10.0);
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