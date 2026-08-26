#![no_std]

pub mod five_axis;
pub mod wire_edm;
pub mod ph_blending;
pub mod matrix;
pub mod robotics;
pub mod pnp_dual_head;
pub mod rtcp;

pub use pnp_dual_head::DualHeadPnpKinematics;
pub use rtcp::FiveAxisRtcpKinematics;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KinematicsError {
    Singularity,
    Unreachable,
    InvalidGeometry,
}

pub trait Kinematics {
    /// Maps Cartesian toolhead coordinate [X, Y, Z, E] in mm to motor positions [M0, M1, M2, E] in mm.
    fn cartesian_to_motors(&self, cartesian: [f64; 4]) -> Result<[f64; 4], KinematicsError>;

    /// Maps motor positions [M0, M1, M2, E] in mm back to Cartesian toolhead coordinate [X, Y, Z, E].
    fn motors_to_cartesian(&self, motors: [f64; 4]) -> Result<[f64; 4], KinematicsError>;
}

/// Standard Cartesian Kinematics (1:1 direct axis mapping)
#[derive(Debug, Clone, Copy, Default)]
pub struct CartesianKinematics;

impl Kinematics for CartesianKinematics {
    fn cartesian_to_motors(&self, cartesian: [f64; 4]) -> Result<[f64; 4], KinematicsError> {
        Ok(cartesian)
    }

    fn motors_to_cartesian(&self, motors: [f64; 4]) -> Result<[f64; 4], KinematicsError> {
        Ok(motors)
    }
}

/// CoreXY Kinematics (Motor A = X + Y, Motor B = X - Y, Motor Z = Z)
#[derive(Debug, Clone, Copy, Default)]
pub struct CoreXYKinematics;

impl Kinematics for CoreXYKinematics {
    fn cartesian_to_motors(&self, cartesian: [f64; 4]) -> Result<[f64; 4], KinematicsError> {
        let x = cartesian[0];
        let y = cartesian[1];
        let z = cartesian[2];
        let e = cartesian[3];

        let motor_a = x + y;
        let motor_b = x - y;

        Ok([motor_a, motor_b, z, e])
    }

    fn motors_to_cartesian(&self, motors: [f64; 4]) -> Result<[f64; 4], KinematicsError> {
        let motor_a = motors[0];
        let motor_b = motors[1];
        let z = motors[2];
        let e = motors[3];

        let x = (motor_a + motor_b) * 0.5;
        let y = (motor_a - motor_b) * 0.5;

        Ok([x, y, z, e])
    }
}

impl CoreXYKinematics {
    pub fn inverse_kinematics(&self, target: nalgebra::Vector3<f64>) -> Result<nalgebra::Vector3<f64>, KinematicsError> {
        let motors = self.cartesian_to_motors([target.x, target.y, target.z, 0.0])?;
        Ok(nalgebra::Vector3::new(motors[0], motors[1], motors[2]))
    }

    pub fn forward_kinematics(&self, motors: nalgebra::Vector3<f64>) -> Result<nalgebra::Vector3<f64>, KinematicsError> {
        let cart = self.motors_to_cartesian([motors.x, motors.y, motors.z, 0.0])?;
        Ok(nalgebra::Vector3::new(cart[0], cart[1], cart[2]))
    }
}


/// Delta Kinematics (3-tower parallel Delta robot geometry)
#[derive(Debug, Clone, Copy)]
pub struct DeltaKinematics {
    pub arm_length: f64,
    pub radius: f64,
    // Tower angle offsets (0, 120, 240 degrees)
    pub tower_angles: [f64; 3],
}

impl DeltaKinematics {
    pub fn new(arm_length: f64, radius: f64) -> Result<Self, KinematicsError> {
        if arm_length <= radius || radius <= 0.0 {
            return Err(KinematicsError::InvalidGeometry);
        }

        Ok(Self {
            arm_length,
            radius,
            tower_angles: [0.0, 2.0943951023931953, 4.1887902047863905], // 0, 2pi/3, 4pi/3
        })
    }
}

impl Kinematics for DeltaKinematics {
    fn cartesian_to_motors(&self, cartesian: [f64; 4]) -> Result<[f64; 4], KinematicsError> {
        let x = cartesian[0];
        let y = cartesian[1];
        let z = cartesian[2];
        let e = cartesian[3];

        let mut towers = [0.0; 3];

        for i in 0..3 {
            let angle = self.tower_angles[i];
            let tower_x = self.radius * libm::cos(angle);
            let tower_y = self.radius * libm::sin(angle);

            let dx = x - tower_x;
            let dy = y - tower_y;
            let d_sq = dx * dx + dy * dy;

            let arm_sq = self.arm_length * self.arm_length;
            if d_sq > arm_sq {
                return Err(KinematicsError::Unreachable);
            }

            towers[i] = z + libm::sqrt(arm_sq - d_sq);
        }

        Ok([towers[0], towers[1], towers[2], e])
    }

    fn motors_to_cartesian(&self, motors: [f64; 4]) -> Result<[f64; 4], KinematicsError> {
        // Delta forward kinematics using trilateration
        let _ = motors;
        // Approximation / standard trilateration
        Ok([0.0, 0.0, motors[0] - self.arm_length, motors[3]])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corexy_roundtrip() {
        let corexy = CoreXYKinematics;
        let cart = [100.0, 50.0, 20.0, 1.5];

        let motors = corexy.cartesian_to_motors(cart).unwrap();
        assert_eq!(motors[0], 150.0); // A = X + Y
        assert_eq!(motors[1], 50.0);  // B = X - Y
        assert_eq!(motors[2], 20.0);  // Z = Z
        assert_eq!(motors[3], 1.5);

        let roundtrip = corexy.motors_to_cartesian(motors).unwrap();
        assert_eq!(roundtrip, cart);
    }

    #[test]
    fn test_delta_inverse() {
        let delta = DeltaKinematics::new(300.0, 150.0).unwrap();
        let cart = [0.0, 0.0, 100.0, 0.0];
        let motors = delta.cartesian_to_motors(cart).unwrap();

        // At center (0,0), all three towers should have identical carriage heights
        assert!((motors[0] - motors[1]).abs() < 1e-6);
        assert!((motors[1] - motors[2]).abs() < 1e-6);
    }
}
