#![no_std]
#![cfg_attr(feature = "async", feature(async_fn_in_trait))]


pub mod adc;
pub mod can;
pub mod dma;
pub mod gpio;
pub mod pwm;
pub mod spi;
pub mod timer;
pub mod uart;

pub mod traits;
pub mod capabilities;
pub mod timer_abstraction;
pub mod dma_abstraction;

pub mod stm32;
pub mod nxp;
pub mod microchip;
pub mod ti;
pub mod emerging;
pub mod infineon;
pub mod renesas;
