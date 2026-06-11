#![no_std]
#![no_main]

// Crate modules
pub mod adc;
#[path = "../boards/mod.rs"]
pub mod boards;
pub mod clock_sync;
pub mod embassy_main;
pub mod fixed_point;
pub mod heater;
pub mod proto_bridge;
pub mod safety;
pub mod stepper;
pub mod transport;

// Panic handler & logging
use panic_probe as _;
use defmt_rtt as _;