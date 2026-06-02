#![deny(clippy::all)]
#![deny(warnings)]

//! A zero-copy, streaming Klipper protocol parser.

use crate::{
    commands::{Command, CommandQueueStep, Message},
    crc::crc16_ccitt,
    Error,
};
use nom::{
    bytes::streaming::take,
    combinator::map,
    number::streaming::{be_u16, be_u32, be_i16, u8},
    sequence::tuple,
    IResult,
};

/// The sync byte that marks the beginning of every Klipper message.
pub const SYNC_BYTE: u8 = 0x1d;

/// A parser for Klipper's binary protocol.
///
/// This parser is designed to be zero-copy and suitable for `no_std` environments.
/// It processes a byte slice and, if a complete and valid message is found,
/// returns a `Message` and the number of bytes consumed.
#[derive(Debug, Default)]
pub struct Parser;

impl Parser {
    /// Creates a new `Parser`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Attempts to parse a single message from the start of the given buffer.
    ///
    /// The method first searches for the `SYNC_BYTE`. If found, it attempts to
    /// parse a complete message frame.
    ///
    /// # Returns
    ///
    /// - `Ok(Some((message, consumed_bytes)))` if a valid message was parsed.
    /// - `Ok(None)` if the buffer does not contain a complete message (e.g.,
    ///   waiting for more data).
    /// - `Err(Error)` if a recoverable error occurred (like a bad CRC), along
    ///   with the number of bytes to discard. The caller should advance their
    ///   buffer by this amount and try again.
    pub fn parse(&self, input: &[u8]) -> Result<Option<(Message, usize)>, (Error, usize)> {
        // Find the first sync byte
        if let Some(sync_pos) = input.iter().position(|&b| b == SYNC_BYTE) {
            let buffer = &input[sync_pos..];
            let original_len = buffer.len();

            // The header must contain at least SYNC_BYTE and LEN byte
            if original_len < 2 {
                return Ok(None);
            }

            let msg_len = buffer[1] as usize; // Length of fields after the length byte
            let expected_total_len = msg_len + 2; // SYNC_BYTE (1) + LEN (1) + msg_len

            if original_len < expected_total_len {
                return Ok(None); // Incomplete message
            }

            // Slice out the full frame (including sync and length)
            let frame = &buffer[..expected_total_len];
            
            // The message block starts at index 1 (the LEN byte)
            let msg_block = &frame[1..]; // Length is msg_len + 1
            
            let received_crc = u16::from_be_bytes(msg_block[msg_block.len() - 2..].try_into().unwrap());
            let calculated_crc = crc16_ccitt(msg_block, msg_len - 1);

            if received_crc != calculated_crc {
                // CRC mismatch. Discard the sync byte and try again.
                return Err((Error::InvalidCrc, sync_pos + 1));
            }

            // The command payload starts at index 2 of msg_block (after LEN and SEQ)
            let cmd_block = &msg_block[2..msg_block.len() - 2];

            // CRC is valid, now parse the command
            match parse_command(cmd_block) {
                Ok((_rem, command)) => {
                    let consumed = sync_pos + expected_total_len;
                    Ok(Some((Message::Command(command), consumed)))
                }
                Err(_) => {
                    Err((Error::InvalidPayload, sync_pos + 1))
                }
            }
        } else {
            // No sync byte found, discard the whole buffer
            Err((Error::InvalidSync, input.len()))
        }
    }
}

/// Uses `nom` to parse the payload into a specific command.
fn parse_command(input: &[u8]) -> IResult<&[u8], Command> {
    let (i, command_id) = u8(input)?;
    match command_id {
        0x01 => map(take(input.len() - 1), |s: &[u8]| {
            let mut dict_version = heapless::Vec::new();
            let _ = dict_version.extend_from_slice(s);
            Command::Identify { dict_version }
        })(i),
        0x02 => Ok((i, Command::GetConfig)),
        0x03 => Ok((i, Command::GetStatus)),
        0x10 => map(
            tuple((be_u32, be_u16, be_i16)),
            |(interval, count, add)| {
                Command::QueueStep(CommandQueueStep {
                    interval_ticks: interval,
                    count,
                    add,
                })
            },
        )(i),
        0x21 => map(tuple((u8, u8)), |(pin, value)| Command::SetDigitalOut {
            pin,
            value,
        })(i),
        0x20 => map(tuple((u8, be_u16)), |(pin, value)| Command::SetPwmOut {
            pin,
            value,
        })(i),
        _ => {
            let mut unknown_payload = heapless::Vec::new();
            let _ = unknown_payload.extend_from_slice(i);
            Ok((&[], Command::Unknown(command_id, unknown_payload)))
        }
    }
}
