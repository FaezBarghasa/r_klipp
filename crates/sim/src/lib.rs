//! # In-Process Simulation Harness
//!
//! This library provides a deterministic simulation environment for emulating
//! microcontroller (MCU) behavior. It is designed to facilitate robust CI testing
//! for host and firmware integration without requiring physical hardware.
//!
//! The two primary components are `SimMcu` and `SimHost`. `SimMcu` simulates the
//! MCU's protocol responses, while `SimHost` provides utilities for running a
//! host binary or in-process component and capturing communication traces.

pub mod fake_mcu;
pub mod harness;

pub use fake_mcu::SimMcu;
pub use harness::{SimHost, TraceEntry};
