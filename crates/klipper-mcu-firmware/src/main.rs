// File: crates/klipper-mcu-firmware/src/main.rs
#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use panic_probe as _;

// --- Memory Pool for a common object (e.g., MoveSegment) ---
// This demonstrates static memory management to avoid heap allocations in real-time tasks.
use motion::planner::MoveSegment;
use core::mem::MaybeUninit;

const MOVE_POOL_SIZE: usize = 16;
static mut MOVE_POOL: [MaybeUninit<MoveSegment>; MOVE_POOL_SIZE] = [MaybeUninit::uninit(); MOVE_POOL_SIZE];
// In a real implementation, a proper allocator would manage this pool.

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

