//! 5-axis kinematic singularity detection and avoidance.
#![no_std]

use micromath::F32Ext;

/// A simple representation of a 5-axis machine's state (A and C axes).
#[derive(Debug, Clone, Copy)]
pub struct FiveAxisState {
    pub a: f32, // Rotation around X
    pub c: f32, // Rotation around Z
}

/// A task for detecting and avoiding kinematic singularities.
pub struct SingularityAvoider {
    /// The threshold (in radians) near the pole (C-axis lock) to trigger avoidance.
    singularity_threshold: f32,
}

impl SingularityAvoider {
    pub fn new(singularity_threshold_degrees: f32) -> Self {
        Self {
            singularity_threshold: singularity_threshold_degrees.to_radians(),
        }
    }

    /// Checks if the machine is near a singularity (gimbal lock).
    /// For a typical 5-axis machine, this occurs when the C-axis is near 0 or PI radians.
    pub fn is_near_singularity(&self, state: &FiveAxisState) -> bool {
        state.c.abs() < self.singularity_threshold || (state.c.abs() - core::f32::consts::PI).abs() < self.singularity_threshold
    }

    /// Attempts to avoid a singularity by redistributing rotation.
    ///
    /// # Arguments
    /// * `current`: The current 5-axis state.
    /// * `target`: The desired next 5-axis state.
    ///
    /// # Returns
    /// A new `FiveAxisState` that avoids the singularity, or the original `target` if no avoidance is needed.
    pub fn avoid_singularity(&self, current: &FiveAxisState, target: &FiveAxisState) -> FiveAxisState {
        if !self.is_near_singularity(target) {
            return *target;
        }

        // If near a singularity, we can "flip" the A-axis and adjust the C-axis
        // to achieve the same tool orientation via a different machine configuration.
        // This is a simplified avoidance strategy. A real implementation would be
        // tightly coupled with the machine's specific kinematics.

        let mut new_target = *target;

        // Flip the A-axis by 180 degrees
        new_target.a = new_target.a + core::f32::consts::PI;

        // Adjust C-axis to compensate.
        // This logic depends heavily on the machine's kinematic model.
        // For this example, we'll assume a simple relationship.
        new_target.c = new_target.c + core::f32::consts::PI;

        // Normalize angles to be within [-PI, PI]
        new_target.a = new_target.a.rem_euclid(2.0 * core::f32::consts::PI);
        if new_target.a > core::f32::consts::PI {
            new_target.a -= 2.0 * core::f32::consts::PI;
        }
        new_target.c = new_target.c.rem_euclid(2.0 * core::f32::consts::PI);
        if new_target.c > core::f32::consts::PI {
            new_target.c -= 2.0 * core::f32::consts::PI;
        }

        new_target
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_singularity_detection() {
        let avoider = SingularityAvoider::new(5.0); // 5 degree threshold

        // Not near singularity
        let state1 = FiveAxisState { a: 0.0, c: 1.0 };
        assert!(!avoider.is_near_singularity(&state1));

        // Near C=0 singularity
        let state2 = FiveAxisState { a: 0.0, c: 0.05 }; // approx 2.8 degrees
        assert!(avoider.is_near_singularity(&state2));

        // Near C=pi singularity
        let state3 = FiveAxisState { a: 0.0, c: 3.1 }; // approx 177 degrees
        assert!(avoider.is_near_singularity(&state3));
    }

    #[test]
    fn test_singularity_avoidance_logic() {
        let avoider = SingularityAvoider::new(5.0);
        let current = FiveAxisState { a: 0.5, c: 0.5 };

        // A target that is near a singularity
        let target_singular = FiveAxisState { a: 1.0, c: 0.01 };
        assert!(avoider.is_near_singularity(&target_singular));

        let new_target = avoider.avoid_singularity(&current, &target_singular);

        // The new target should no longer be near the singularity
        assert!(!avoider.is_near_singularity(&new_target));

        // And the A-axis should have been flipped
        assert!((new_target.a - (target_singular.a + core::f32::consts::PI)).abs() < 1e-4);
    }

    #[test]
    fn test_no_avoidance_when_not_near_singularity() {
        let avoider = SingularityAvoider::new(5.0);
        let current = FiveAxisState { a: 0.5, c: 0.5 };
        let target_safe = FiveAxisState { a: 1.0, c: 1.0 };

        let new_target = avoider.avoid_singularity(&current, &target_safe);

        // Should return the original target
        assert!((new_target.a - target_safe.a).abs() < 1e-6);
        assert!((new_target.c - target_safe.c).abs() < 1e-6);
    }
}
