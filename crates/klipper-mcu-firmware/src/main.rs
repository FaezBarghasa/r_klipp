// File: crates/klipper-mcu-firmware/src/main.rs
#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _; // global logger
use panic_probe as _;

// --- Memory Pool for MoveSegment ---
// The firmware uses a static memory pool for `MoveSegment` objects to avoid dynamic memory allocation (heap)
// during real-time operations. This is crucial for maintaining predictable performance and preventing
// memory fragmentation.
//
// `MOVE_POOL` is a statically allocated array of `MaybeUninit<MoveSegment>`, allowing for unsafe, direct
// memory management. The size of the pool is defined by `MOVE_POOL_SIZE`.
//
// A proper memory manager or allocator would typically be responsible for initializing, allocating,
// and deallocating from this pool to ensure memory safety and prevent data races.
use motion::planner::MoveSegment;
use core::mem::MaybeUninit;

const MOVE_POOL_SIZE: usize = 16;
static mut MOVE_POOL: [MaybeUninit<MoveSegment>; MOVE_POOL_SIZE] = [MaybeUninit::uninit(); MOVE_POOL_SIZE];

// Application Modules
pub mod adc;
pub mod heater;
pub mod proto_bridge;
pub mod safety;
pub mod stepper;
pub mod fixed_point;

// --- Runtime Selection ---
// The firmware is designed to be runtime-agnostic and supports multiple real-time schedulers.
// The choice of runtime is determined by feature flags passed at compile time. This allows
// developers to select the scheduler that best fits their needs or the target hardware.
//
// - `embassy-rt`: Uses the Embassy framework, which provides an async/await runtime.
// - `rtic-rt`: Uses the Real-Time Interrupt-driven Concurrency (RTIC) framework.
//
// The corresponding module (`embassy_main.rs` or `rtic_main.rs`) contains the main
// application logic and task definitions for that specific runtime.

#[cfg(feature = "embassy-rt")]
mod embassy_main;
#[cfg(feature = "embassy-rt")]
use embassy_main as _;

#[cfg(feature = "rtic-rt")]
mod rtic_main;
#[cfg(feature = "rtic-rt")]
use rtic_main as _;

