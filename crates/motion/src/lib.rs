//! # Motion Control Crate
//!
//! A deterministic motion planner and kinematics library usable on both host
//! and MCU (`no_std`-friendly).
//!
//! Provides core motion primitives:
//! - Kinematics (Cartesian, CoreXY)
//! - Trapezoidal velocity profile planner
//! - Step event generation for the stepper driver
//! - Hooks for features like Pressure Advance

#![cfg_attr(not(feature = "std"), no_std)]
#![cfg_attr(not(feature = "std"), feature(const_fn_floating_point_arithmetic))]


#[cfg(feature = "alloc")]
extern crate alloc;

pub mod errors;
pub mod kinematics;
pub mod planner;
pub mod profile;

// Re-export core types for easier access
pub use errors::PlannerError;
pub use kinematics::{CoreXYKinematics, CartesianKinematics, Kinematics, StepperAxis};
pub use planner::{MoveSegment, MotionPlanner};
pub use profile::PressureAdvance;
use mcu_drivers::stepper::StepCommand;

/// A point in 3D cartesian space.
/// Units are typically in millimeters.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct CartesianPoint {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
