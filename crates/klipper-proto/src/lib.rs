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
//!   or errors from malformed input.

#[cfg(feature = "alloc")]
extern crate alloc;

pub mod commands;
pub mod crc;
pub mod parser;
pub mod autoconfig;

#[cfg(feature = "std")]
pub mod codec;
#[cfg(feature = "std")]
pub mod io;
#[cfg(any(feature = "std", feature = "alloc"))]
pub mod registry;

/// Common error type for the protocol crate.
#[derive(Debug)]
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
    /// An incomplete frame or data chunk.
    Incomplete,
    /// A serialization or deserialization error from postcard.
    Postcard(postcard::Error),
    /// An I/O error.
    #[cfg(feature = "std")]
    Io(std::io::Error),
}

impl PartialEq for Error {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Error::IncompleteFrame, Error::IncompleteFrame) => true,
            (Error::InvalidSync, Error::InvalidSync) => true,
            (Error::InvalidCrc, Error::InvalidCrc) => true,
            (Error::InvalidPayload, Error::InvalidPayload) => true,
            (Error::BufferTooSmall, Error::BufferTooSmall) => true,
            (Error::Incomplete, Error::Incomplete) => true,
            (Error::Postcard(e1), Error::Postcard(e2)) => e1 == e2,
            #[cfg(feature = "std")]
            (Error::Io(e1), Error::Io(e2)) => e1.kind() == e2.kind(),
            _ => false,
        }
    }
}

impl Eq for Error {}

impl From<postcard::Error> for Error {
    fn from(err: postcard::Error) -> Self {
        Error::Postcard(err)
    }
}

#[cfg(feature = "std")]
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[cfg(feature = "std")]
impl std::error::Error for Error {}

#[cfg(feature = "std")]
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}
