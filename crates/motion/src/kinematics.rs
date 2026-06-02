//! Kinematics traits and implementations.

#[cfg(not(feature = "std"))]
use libm::{sqrt, fmax};
#[cfg(feature = "std")]
fn sqrt(x: f64) -> f64 {
    x.sqrt()
}

use crate::CartesianPoint;

/// Identifiers for the physical stepper axes.
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde-support", derive(serde::Serialize, serde::Deserialize))]
pub enum StepperAxis {
    X,
    Y,
    Z,
    A, // CoreXY A stepper
    B, // CoreXY B stepper
}

/// A trait for kinematic models that translate cartesian moves to stepper positions.
pub trait Kinematics {
    /// The number of axes in this kinematic model.
    const AXES: usize;

    /// Calculate the stepper motor positions for a given Cartesian coordinate.
    ///
    /// # Arguments
    /// * `point` - The target `CartesianPoint` (x, y, z).
    /// * `out` - The output slice of size at least `Self::AXES` to write the target positions into.
    fn cartesian_to_stepper_positions(&self, point: CartesianPoint, out: &mut [f64]);

    /// Calculate the total distance a move will travel in stepper space.
    ///
    /// This is used by the planner to calculate the number of steps.
    ///
    /// # Arguments
    /// * `from` - The starting `CartesianPoint`.
    /// * `to` - The ending `CartesianPoint`.
    ///
    /// # Returns
    /// The length of the move vector in stepper-space.
    fn stepper_move_distance(&self, from: CartesianPoint, to: CartesianPoint) -> f64 {
        let mut start_pos = [0.0; 8];
        let mut end_pos = [0.0; 8];
        self.cartesian_to_stepper_positions(from, &mut start_pos[..Self::AXES]);
        self.cartesian_to_stepper_positions(to, &mut end_pos[..Self::AXES]);

        let mut dist_sq = 0.0;
        for i in 0..Self::AXES {
            let delta = end_pos[i] - start_pos[i];
            dist_sq += delta * delta;
        }
        sqrt(dist_sq)
    }
}

// --- Cartesian Kinematics ---

/// Standard Cartesian kinematics (X, Y, Z steppers map directly to X, Y, Z axes).
#[derive(Debug, Default)]
pub struct CartesianKinematics {
    pub steps_per_mm_x: f64,
    pub steps_per_mm_y: f64,
    pub steps_per_mm_z: f64,
}

impl Kinematics for CartesianKinematics {
    const AXES: usize = 3;

    fn cartesian_to_stepper_positions(&self, point: CartesianPoint, out: &mut [f64]) {
        out[0] = point.x as f64 * self.steps_per_mm_x;
        out[1] = point.y as f64 * self.steps_per_mm_y;
        out[2] = point.z as f64 * self.steps_per_mm_z;
    }
}


// --- CoreXY Kinematics ---

/// CoreXY kinematics model.
/// See: https://corexy.com/theory.html
#[derive(Debug, Default)]
pub struct CoreXYKinematics {
    pub steps_per_mm_a: f64, // Typically steps_per_mm for both X and Y
    pub steps_per_mm_b: f64,
    pub steps_per_mm_z: f64,
}

impl Kinematics for CoreXYKinematics {
    const AXES: usize = 3;

    fn cartesian_to_stepper_positions(&self, point: CartesianPoint, out: &mut [f64]) {
        // Z is independent
        out[2] = point.z as f64 * self.steps_per_mm_z;
        // CoreXY transform for A and B steppers
        out[0] = (point.x as f64 + point.y as f64) * self.steps_per_mm_a;
        out[1] = (point.x as f64 - point.y as f64) * self.steps_per_mm_b;
    }
}
