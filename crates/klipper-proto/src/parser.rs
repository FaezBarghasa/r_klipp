#![deny(clippy::all)]
#![deny(warnings)]

//! A zero-copy, streaming Klipper protocol parser.

use crate::{
    commands::{Command, CommandQueueStep, Message},
    crc::crc16_ccitt,
    Error,
};
use nom::{
    bytes::streaming::{tag, take},
    combinator::map,
    error::Error as NomError,
    number::streaming::{be_u16, be_u32, i16, u8},
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
    pub fn parse<'a>(&self, input: &'a [u8]) -> Result<Option<(Message<'a>, usize)>, (Error, usize)> {
        // Find the first sync byte
        if let Some(sync_pos) = input.iter().position(|&b| b == SYNC_BYTE) {
            let buffer = &input[sync_pos..];
            let original_len = buffer.len();

            // The header is 4 bytes: [LEN, SEQ, CMD_ID, CMD_ID] (CMD_ID is u16)
            if original_len < 4 {
                return Ok(None); // Incomplete header
            }

            let msg_len = buffer[0] as usize;
            if original_len < msg_len {
                return Ok(None); // Incomplete message payload
            }

            // We have a full potential message. Slice it out.
            // The full frame includes [LEN, SEQ, CMD, PAYLOAD, CRC]
            let frame_with_len = &buffer[..msg_len];
            let payload_and_crc = &frame_with_len[1..];
            let payload = &payload_and_crc[..payload_and_crc.len() - 2];
            let received_crc =
                u16::from_be_bytes(payload_and_crc[payload_and_crc.len() - 2..].try_into().unwrap());

            // Validate CRC
            let calculated_crc = crc16_ccitt(frame_with_len, msg_len - 2);
            if received_crc != calculated_crc {
                // CRC mismatch. Discard the sync byte and try again.
                return Err((Error::InvalidCrc, sync_pos + 1));
            }

            // CRC is valid, now parse the command
            match parse_command(payload) {
                Ok((_rem, command)) => {
                    let consumed = sync_pos + msg_len + 1; // +1 for the sync byte
                    Ok(Some((Message::Command(command), consumed)))
                }
                Err(_) => {
                    // Payload is invalid for any known command. Discard and continue.
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
        0x01 => map(take(input.len() - 1), |s| Command::Identify {
            dict_version: s,
        })(i),
        0x02 => Ok((i, Command::GetConfig)),
        0x03 => Ok((i, Command::GetStatus)),
        0x10 => map(
            tuple((be_u32, be_u16, i16)),
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
        _ => Ok((&[], Command::Unknown(command_id, i))),
    }
}
