//! Klipper protocol framing, compression, and binary encoding/decoding.

use crate::{commands::Message, Error};
use postcard;

#[cfg(feature = "std")]
use bytes::{Buf, BytesMut};
#[cfg(feature = "std")]
use tokio_util::codec::{Decoder, Encoder};

const SYNC_BYTE: u8 = 0x7E;
const ESCAPE_BYTE: u8 = 0x7D;
const MAX_FRAME_SIZE: usize = 256;

/// A codec for encoding and decoding Klipper protocol messages.
///
/// This codec handles message framing, CRC checksums, and Klipper's
/// dictionary-based compression.
#[derive(Debug, Default)]
pub struct KlipperCodec {
    // Klipper's dictionary is typically established at connection time.
    // For this implementation, we use a static default dictionary.
    // A future version could allow for a dynamic dictionary.
}

impl KlipperCodec {
    /// Creates a new `KlipperCodec`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Calculates the 8-bit CRC for a slice of data.
    fn crc8_atm(data: &[u8]) -> u8 {
        let mut crc = 0x00;
        for byte in data {
            crc ^= byte;
            for _ in 0..8 {
                if crc & 0x80 != 0 {
                    crc = (crc << 1) ^ 0x07;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }

    // Note: Klipper's compression is more complex and involves a dynamically
    // built dictionary. This is a simplified placeholder.
    // For now, we will just escape the necessary bytes. A full compression
    // implementation would replace common strings with dictionary indices.

    /// Escapes bytes in the payload for framing.
    fn escape(payload: &[u8], dst: &mut BytesMut) {
        for &byte in payload {
            if byte == SYNC_BYTE || byte == ESCAPE_BYTE {
                dst.extend_from_slice(&[ESCAPE_BYTE, byte ^ 0x20]);
            } else {
                dst.extend_from_slice(&[byte]);
            }
        }
    }
}

#[cfg(feature = "std")]
impl Encoder<Message> for KlipperCodec {
    type Error = Error;

    /// Encodes a `Message` into a byte frame.
    fn encode(&mut self, item: Message, dst: &mut BytesMut) -> Result<(), Self::Error> {
        // 1. Serialize the message using postcard
        let mut temp_buf = [0u8; MAX_FRAME_SIZE];
        let serialized = postcard::to_slice(&item, &mut temp_buf)?;

        // 2. Calculate CRC on the unescaped payload
        let crc = Self::crc8_atm(serialized);

        // 3. Frame the message
        dst.reserve(3 + serialized.len()); // Conservative estimate
        dst.extend_from_slice(&[SYNC_BYTE, 0, crc]); // Length is placeholder for now

        // 4. Escape payload and write to buffer
        Self::escape(serialized, dst);

        // 5. Update the length field
        let frame_len = dst.len() - 1; // -1 for sync byte
        if frame_len > 255 {
            // This should be rare but is a protocol limitation
            return Err(Error::BufferTooSmall);
        }
        dst[1] = frame_len as u8;

        Ok(())
    }
}

#[cfg(feature = "std")]
impl Decoder for KlipperCodec {
    type Item = Message;
    type Error = Error;

    /// Decodes a byte frame into a `Message`.
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Find the start of a frame
        if let Some(sync_pos) = src.iter().position(|&b| b == SYNC_BYTE) {
            src.advance(sync_pos);
        } else {
            src.clear();
            return Ok(None);
        }

        if src.len() < 3 {
            return Ok(None); // Not enough for header
        }

        let frame_len = src[1] as usize;
        if src.len() < 1 + frame_len {
            return Ok(None); // Not enough for full frame
        }

        // We have a full potential frame, let's process it
        let mut frame_data = src.split_to(1 + frame_len);
        let _sync = frame_data.get_u8();
        let _len = frame_data.get_u8();
        let crc = frame_data.get_u8();

        // Unescape the payload
        let mut payload = BytesMut::with_capacity(frame_data.len());
        let mut it = frame_data.iter().peekable();
        while let Some(&byte) = it.next() {
            if byte == ESCAPE_BYTE {
                if let Some(&&next_byte) = it.peek() {
                    it.next(); // consume
                    payload.extend_from_slice(&[next_byte ^ 0x20]);
                } else {
                    return Err(Error::Incomplete); // Dangling escape byte
                }
            } else {
                payload.extend_from_slice(&[byte]);
            }
        }

        // Validate CRC on the unescaped payload
        let calculated_crc = Self::crc8_atm(&payload);
        if crc != calculated_crc {
            // Discard the invalid frame and try again on the next call
            return Err(Error::InvalidCrc);
        }

        // Deserialize the message
        let message: Message = postcard::from_bytes(&payload)?;
        Ok(Some(message))
    }
}

