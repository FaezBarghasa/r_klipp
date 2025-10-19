#![cfg_attr(not(feature = "std"), no_std)]

//! # Thermal Management Crate
//!
//! `thermal` provides a suite of tools for handling thermal systems, particularly
//! for embedded devices like 3D printers or other robotics, but is also usable
//! on host systems for simulation and testing.
//!
//! ## Features
//!
//! - **Thermistor Models**: Convert ADC readings to temperature using lookup tables
//!   or polynomial fits.
//! - **Signal Filtering**: Low-pass and moving average filters to reduce noise from ADC readings.
//! - **PID Controller**: A PID controller with anti-windup for precise temperature control.
//! - **Heater Safety**: Logic for detecting thermal runaway and managing heater shutdowns.
//! - **`no_std` support**: Can be used in bare-metal environments.
//! - **Optional `serde` support**: For serializing/deserializing thermal components.

// Re-export the main components for easier access.
pub mod filter;
pub mod pid;
pub mod safety;
pub mod thermistor;

// Re-export key types
pub use filter::{Filter, LowPassFilter, MovingAverageFilter};
pub use pid::Pid;
pub use safety::{HeaterSafety, SafetyLimits, ThermalState};
pub use thermistor::{SteinhartHart, Thermistor};
