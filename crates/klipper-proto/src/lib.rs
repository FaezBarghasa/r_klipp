#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod autoconfig;
pub mod clock_sync;
pub mod codec;
pub mod connection;
pub mod commands;
pub mod crc;
pub mod io;
pub mod parser;
pub mod registry;
pub mod feeder;

pub use clock_sync::{ClockSample, DpllClockSync};
pub use connection::{ConnectionManager, ConnectionState};
pub use feeder::{FeederCommand, FeederResponse, FeederStatus, FeederCommandType};