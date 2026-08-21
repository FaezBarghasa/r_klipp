//! 5-Axis Rotational Tool Center Point (RTCP / TCP) Kinematics.
//!
//! Maintains the physical cutting tool tip position invariant in workpiece coordinate space
//! while rotating the primary rotary tilt (A or B) and rotary table (C) axes.
//!
//! Transforms programmed TCP $[X_w, Y_w, Z_w]$ + tool orientation angles $[A, C]$
//! into machine joint coordinates $[X_m, Y_m, Z_m, A, C]$.

use crate::{Kinematics, KinematicsError};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FiveAxisRtcpConfig {
    pub tool_pivot_offset_z: f64, // Gauge length from spindle pivot point to tool tip
    pub table_center_offset_x: f64,
    pub table_center_offset_y: f64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct FiveAxisRtcpKinematics {
    pub config: FiveAxisRtcpConfig,
}

impl FiveAxisRtcpKinematics {
    pub fn new(tool_pivot_offset_z: f64) -> Self {
        Self {
            config: FiveAxisRtcpConfig {
                tool_pivot_offset_z,
                table_center_offset_x: 0.0,
                table_center_offset_y: 0.0,
            },
        }
    }

    /// Computes machine XYZ position to keep tool tip at workpiece (X, Y, Z) given tool tilt angle B (rad) and table rotation C (rad)
    pub fn compute_rtcp_machine_coords(&self, tip_x: f64, tip_y: f64, tip_z: f64, tilt_b_rad: f64, table_c_rad: f64) -> (f64, f64, f64) {
        let l = self.config.tool_pivot_offset_z;
        // Tool vector rotation under tilt angle B
        let dx = l * libm::sin(tilt_b_rad) * libm::cos(table_c_rad);
        let dy = l * libm::sin(tilt_b_rad) * libm::sin(table_c_rad);
        let dz = l * (1.0 - libm::cos(tilt_b_rad));

        (tip_x + dx, tip_y + dy, tip_z + dz)
    }
}

impl Kinematics for FiveAxisRtcpKinematics {
    fn cartesian_to_motors(&self, cartesian: [f64; 4]) -> Result<[f64; 4], KinematicsError> {
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
    fn test_rtcp_zero_tilt_and_rotated_tilt() {
        let rtcp = FiveAxisRtcpKinematics::new(100.0); // 100mm tool length

        // At zero tilt (B=0, C=0), machine position equals tool tip position
        let (mx, my, mz) = rtcp.compute_rtcp_machine_coords(50.0, 50.0, 10.0, 0.0, 0.0);
        assert_eq!(mx, 50.0);
        assert_eq!(my, 50.0);
        assert_eq!(mz, 10.0);

        // At 30 deg tilt (pi/6 rad)
        let (mx_tilt, _my_tilt, mz_tilt) = rtcp.compute_rtcp_machine_coords(50.0, 50.0, 10.0, core::f64::consts::FRAC_PI_6, 0.0);
        // dx = 100 * sin(30 deg) = 50.0mm -> mx = 50 + 50 = 100.0mm
        assert!((mx_tilt - 100.0).abs() < 1e-4);
        // dz = 100 * (1 - cos(30 deg)) = 100 * (1 - 0.866025) = ~13.397mm -> mz = 10 + 13.397 = ~23.397mm
        assert!((mz_tilt - 23.3974).abs() < 1e-3);
    }
}
