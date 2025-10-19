//! # Klipper MCU Firmware
//!
//! This is the main crate for the Klipper MCU firmware. It is a `no_std` application
//! that is designed to run on a microcontroller.
//!
//! ## Feature Flags
//!
//! This crate uses feature flags to select the runtime and the target board.
//!
//! *   `embassy-rt`: Use the Embassy async runtime.
//! *   `rtic-rt`: Use the RTIC real-time framework.
//! *   `board-stm32f407`: Build for the STM32F407 board.
//!
//! ## Application Modules
//!
//! The firmware is divided into several modules, each responsible for a specific
//! part of the printer's functionality:
//!
//! *   `adc`: ADC sampling for thermistors.
//! *   `heater`: PID heater control.
//! *   `proto_bridge`: Communication with the Klipper host.
//! *   `safety`: Safety monitoring and emergency stop.
//! *   `stepper`: Stepper motor control.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use panic_probe as _;

// Application Modules
pub mod adc;
pub mod heater;
pub mod proto_bridge;
pub mod safety;
pub mod stepper;
pub mod fixed_point;

#[cfg(feature = "embassy-rt")]
mod embassy_main;
#[cfg(feature = "embassy-rt")]
use embassy_main as _;

#[cfg(feature = "rtic-rt")]
mod rtic_main;
#[cfg(feature = "rtic-rt")]
use rtic_main as _;
