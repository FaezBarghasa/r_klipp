//! Binary streaming parser for framed Klipper and r_klipp protocol packets.

pub const SYNC_BYTE: u8 = 0x7E;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProtoError {
    InvalidSync,
    InvalidCrc,
    Incomplete,
    BufferOverflow,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RawFrame<'a> {
    pub sequence: u8,
    pub command_id: u8,
    pub payload: &'a [u8],
}

pub struct FrameParser;

impl FrameParser {
    pub fn new() -> Self {
        Self
    }

    /// Attempts to parse a single framed packet from `data`.
    /// Returns `Ok(Some((RawFrame, bytes_consumed)))` on success.
    pub fn parse<'a>(&self, data: &'a [u8]) -> Result<Option<(RawFrame<'a>, usize)>, (ProtoError, usize)> {
        if data.is_empty() {
            return Ok(None);
        }

        // Find sync byte
        let sync_pos = match data.iter().position(|&b| b == SYNC_BYTE) {
            Some(pos) => pos,
            None => return Err((ProtoError::InvalidSync, data.len())),
        };

        let frame_data = &data[sync_pos..];
        if frame_data.len() < 5 {
            // Need at least: SYNC, LEN, SEQ, CMD, CRC16_HI, CRC16_LO
            return Ok(None);
        }

        let msg_len = frame_data[1] as usize;
        let total_frame_len = msg_len + 2; // + SYNC + LEN

        if frame_data.len() < total_frame_len {
            return Ok(None); // Incomplete frame
        }

        let seq = frame_data[2];
        let cmd_id = frame_data[3];
        let payload_len = msg_len.saturating_sub(4); // minus seq, cmd_id, crc16 (2 bytes)
        let payload = &frame_data[4..4 + payload_len];

        // CRC check over frame payload
        let expected_crc = u16::from_be_bytes([
            frame_data[total_frame_len - 2],
            frame_data[total_frame_len - 1],
        ]);

        let computed_crc = crate::crc::crc16_ccitt(&frame_data[1..total_frame_len - 2], total_frame_len - 3);
        if expected_crc != computed_crc {
            return Err((ProtoError::InvalidCrc, sync_pos + 1));
        }

        Ok(Some((RawFrame { sequence: seq, command_id: cmd_id, payload }, sync_pos + total_frame_len)))
    }
}