//! Dual-Head Pick & Place (PnP / PIP) Kinematics.
//!
//! Controls a shared XY gantry carrying two independent vertical Z-axes (`head_1_z`, `head_2_z`)
//! and dual rotational C-axes (`head_1_c`, `head_2_c`) with non-collision constraints.

use crate::{Kinematics, KinematicsError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PnpToolheadState {
    pub x: f64,
    pub y: f64,
    pub z1: f64,
    pub z2: f64,
    pub c1: f64, // Rotation angle head 1 (degrees)
    pub c2: f64, // Rotation angle head 2 (degrees)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DualHeadPnpKinematics {
    pub head_offset_x: f64, // Physical distance between nozzle 1 and nozzle 2 in X
    pub head_offset_y: f64, // Physical distance in Y
    pub z_safe_clearance_mm: f64,
}

impl DualHeadPnpKinematics {
    pub fn new(head_offset_x: f64, head_offset_y: f64) -> Self {
        Self {
            head_offset_x,
            head_offset_y,
            z_safe_clearance_mm: 5.0,
        }
    }

    /// Computes the physical coordinates of Head 2 given Gantry XY position
    pub fn head_2_pos(&self, gantry_x: f64, gantry_y: f64) -> (f64, f64) {
        (gantry_x + self.head_offset_x, gantry_y + self.head_offset_y)
    }

    /// Validates that both heads do not collide when descending simultaneously
    pub fn validate_head_safety(&self, z1: f64, z2: f64) -> Result<(), KinematicsError> {
        // In SMT PnP, picking and placing requires retracting one nozzle when the other is low
        if z1 < self.z_safe_clearance_mm && z2 < self.z_safe_clearance_mm {
            // Both heads extended low simultaneously: collision/interference warning
            return Err(KinematicsError::Singularity);
        }
        Ok(())
    }
}

impl Kinematics for DualHeadPnpKinematics {
    fn cartesian_to_motors(&self, cartesian: [f64; 4]) -> Result<[f64; 4], KinematicsError> {
        // Standard Cartesian mapping for gantry X, Y, and active toolhead Z
        Ok(cartesian)
    }

    fn motors_to_cartesian(&self, motors: [f64; 4]) -> Result<[f64; 4], KinematicsError> {
        Ok(motors)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dual_head_offset_and_safety() {
        let pnp = DualHeadPnpKinematics::new(40.0, 0.0);
        let (h2_x, h2_y) = pnp.head_2_pos(100.0, 50.0);
        assert_eq!(h2_x, 140.0);
        assert_eq!(h2_y, 50.0);

        // Safe: head 1 down (0mm), head 2 retracted (15mm)
        assert!(pnp.validate_head_safety(0.0, 15.0).is_ok());

        // Dangerous: both heads down at 1.0mm
        assert!(pnp.validate_head_safety(1.0, 1.0).is_err());
    }
}
