//! Parser tests for binary framed protocol.

use klipper_proto::parser::{FrameParser, ProtoError, SYNC_BYTE};
use klipper_proto::crc::crc16_ccitt;

fn build_test_frame(seq: u8, command_id: u8, payload: &[u8]) -> Vec<u8> {
    let msg_len = 1 /* seq */ + 1 /* cmd_id */ + payload.len() + 2 /* crc */;
    let mut frame = Vec::with_capacity(msg_len + 2);
    frame.push(SYNC_BYTE);
    frame.push(msg_len as u8);
    frame.push(seq);
    frame.push(command_id);
    frame.extend_from_slice(payload);

    let crc = crc16_ccitt(&frame[1..], msg_len - 1);
    frame.extend_from_slice(&crc.to_be_bytes());

    frame
}

#[test]
fn test_parse_valid_frame() {
    let frame = build_test_frame(1, 0x02, &[0x10, 0x20]);
    let parser = FrameParser::new();

    let (raw, consumed) = parser.parse(&frame).unwrap().unwrap();
    assert_eq!(consumed, frame.len());
    assert_eq!(raw.sequence, 1);
    assert_eq!(raw.command_id, 0x02);
    assert_eq!(raw.payload, &[0x10, 0x20]);
}

#[test]
fn test_parse_with_leading_noise() {
    let frame = build_test_frame(3, 0x03, &[]);
    let mut buffer = vec![0x00, 0x01, 0x02, 0x03]; // Noise
    buffer.extend_from_slice(&frame);
    let parser = FrameParser::new();

    let (raw, consumed) = parser.parse(&buffer).unwrap().unwrap();
    assert_eq!(consumed, buffer.len());
    assert_eq!(raw.sequence, 3);
    assert_eq!(raw.command_id, 0x03);
}

#[test]
fn test_bad_crc_error() {
    let mut frame = build_test_frame(7, 0x02, &[]);
    let last = frame.len() - 1;
    frame[last] ^= 0xFF; // Corrupt CRC

    let parser = FrameParser::new();
    let err = parser.parse(&frame).unwrap_err();
    assert_eq!(err.0, ProtoError::InvalidCrc);
}
