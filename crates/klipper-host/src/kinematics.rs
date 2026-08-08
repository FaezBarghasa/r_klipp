//! Motion and Kinematics
//!
//! This module is responsible for translating high-level G-code moves (e.g., "move to X, Y")
//! into low-level step commands for the printer's motors. This is a placeholder
//! for a more complex motion planning system.

use crate::state::Position;
use tracing::info;

/// A simplified representation of a motor step.
#[derive(Debug)]
pub struct Step {
    pub motor: String,
    pub steps: i32,
}

/// A trait for different types of printer kinematics (e.g., Cartesian, CoreXY, Delta).
pub trait Kinematics {
    /// Calculates the steps required to move from one position to another.
    fn calculate_move(&self, from: &Position, to: &Position) -> Vec<Step>;
}

/// A basic implementation for a Cartesian printer.
pub struct CartesianKinematics {
    pub steps_per_mm_x: f32,
    pub steps_per_mm_y: f32,
    pub steps_per_mm_z: f32,
}

impl Kinematics for CartesianKinematics {
    /// For a Cartesian printer, the calculation is straightforward.
    fn calculate_move(&self, from: &Position, to: &Position) -> Vec<Step> {
        info!("Calculating move from {:?} to {:?}", from, to);
        let mut steps = Vec::new();

        let dx = to.x - from.x;
        let dy = to.y - from.y;
        let dz = to.z - from.z;

        if dx.abs() > f32::EPSILON {
            steps.push(Step {
                motor: "stepper_x".to_string(),
                steps: (dx * self.steps_per_mm_x).round() as i32,
            });
        }
        if dy.abs() > f32::EPSILON {
            steps.push(Step {
                motor: "stepper_y".to_string(),
                steps: (dy * self.steps_per_mm_y).round() as i32,
            });
        }
        if dz.abs() > f32::EPSILON {
            steps.push(Step {
                motor: "stepper_z".to_string(),
                steps: (dz * self.steps_per_mm_z).round() as i32,
            });
        }

        steps
    }
}
