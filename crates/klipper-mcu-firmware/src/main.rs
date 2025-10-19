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

#[cfg(feature = "embassy-rt")]
mod embassy_main;
#[cfg(feature = "embassy-rt")]
use embassy_main as _;

#[cfg(feature = "rtic-rt")]
mod rtic_main;
#[cfg(feature = "rtic-rt")]
use rtic_main as _;
