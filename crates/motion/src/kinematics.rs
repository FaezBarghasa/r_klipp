//! Kinematics traits and implementations.

#[cfg(not(feature = "std"))]
use libm::{sqrt, fmax};
#[cfg(feature = "std")]
use std::f64::{sqrt, NAN};

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
    ///
    /// # Returns
    /// An array of f64 where each element corresponds to a stepper's target position.
    /// The order of axes is implementation-defined.
    fn cartesian_to_stepper_positions(&self, point: CartesianPoint) -> [f64; Self::AXES];

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
        let start_pos = self.cartesian_to_stepper_positions(from);
        let end_pos = self.cartesian_to_stepper_positions(to);

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

    fn cartesian_to_stepper_positions(&self, point: CartesianPoint) -> [f64; Self::AXES] {
        [
            point.x * self.steps_per_mm_x,
            point.y * self.steps_per_mm_y,
            point.z * self.steps_per_mm_z,
        ]
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

    fn cartesian_to_stepper_positions(&self, point: CartesianPoint) -> [f64; Self::AXES] {
        // Z is independent
        let z_pos = point.z * self.steps_per_mm_z;
        // CoreXY transform for A and B steppers
        let a_pos = (point.x + point.y) * self.steps_per_mm_a;
        let b_pos = (point.x - point.y) * self.steps_per_mm_b;
        [a_pos, b_pos, z_pos]
    }
}
