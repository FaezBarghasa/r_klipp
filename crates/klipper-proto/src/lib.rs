#![deny(clippy::all)]
#![deny(warnings)]
#![cfg_attr(not(feature = "std"), no_std)]

//! # Klipper MCU Protocol
//!
//! A `no_std` zero-copy implementation of the Klipper MCU binary protocol.
//! This crate provides a high-performance parser for decoding messages from a
//! host and definitions for all protocol commands and responses.
//!
//! ## Key Components
//!
//! - **[`parser`]**: A zero-copy `Parser` that can decode messages from a byte
//!   stream, designed for use in interrupt handlers or async tasks.
//! - **[`commands`]**: Typed enums for all `Command`s and `Response`s in the
//!   protocol.
//! - **[`crc`]**: A `const fn` implementation of the CRC16-CCITT algorithm used
//!   for message integrity.
//!
//! ## Design
//!
//! The parser is designed for performance and robustness:
//!
//! - **Zero-Copy**: It operates on `&[u8]` slices and avoids heap allocations.
//! - **Streaming**: It can handle partial messages, finding frame boundaries
//!   in a continuous stream of bytes.
//! - **Fuzzing-Resistant**: Robust checks for length and CRC prevent panics
-//!   or errors from malformed input.

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod commands;
pub mod crc;
pub mod parser;

/// Common error type for the protocol crate.
#[derive(Debug, PartialEq, Eq)]
pub enum Error {
    /// A frame was received that was not long enough to be valid.
    IncompleteFrame,
    /// An invalid frame sync byte (0x1D) was expected but not found.
    InvalidSync,
    /// The CRC-16 checksum of a received frame was invalid.
    InvalidCrc,
    /// The message payload could not be deserialized into a known command.
    InvalidPayload,
    /// A buffer was too small for the requested operation.
    BufferTooSmall,
}
