#![cfg(test)]

use klipper_proto::{
    commands::{Command, CommandQueueStep},
    crc::crc16_ccitt,
    parser::{Parser, SYNC_BYTE},
};

/// Helper to construct a valid Klipper message frame for testing.
fn build_test_frame(seq: u8, command_id: u8, payload: &[u8]) -> Vec<u8> {
    let msg_len = 1 (seq) + 1 (cmd_id) + payload.len() + 2 (crc);
    let mut frame = Vec::with_capacity(msg_len + 1);
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
fn test_parse_get_config() {
    let frame = build_test_frame(1, 0x02, &[]);
    let parser = Parser::new();

    let result = parser.parse(&frame);
    assert!(result.is_ok());
    let (msg, consumed) = result.unwrap().unwrap();

    assert_eq!(consumed, frame.len());
    match msg {
        klipper_proto::commands::Message::Command(Command::GetConfig) => { /* Success */ }
        _ => panic!("Incorrect message parsed"),
    }
}

#[test]
fn test_parse_queue_step() {
    let payload = [
        0x00, 0x00, 0x27, 0x10, // interval = 10000
        0x00, 0x01, // count = 1
        0x00, 0x00, // add = 0
    ];
    let frame = build_test_frame(2, 0x10, &payload);
    let parser = Parser::new();

    let (msg, consumed) = parser.parse(&frame).unwrap().unwrap();
    assert_eq!(consumed, frame.len());

    match msg {
        klipper_proto::commands::Message::Command(Command::QueueStep(qs)) => {
            assert_eq!(
                qs,
                CommandQueueStep {
                    interval_ticks: 10000,
                    count: 1,
                    add: 0
                }
            );
        }
        _ => panic!("Incorrect message parsed"),
    }
}

#[test]
fn test_parse_with_leading_noise() {
    let frame = build_test_frame(3, 0x03, &[]); // GetStatus
    let mut buffer = vec![0x00, 0x01, 0x02, 0x03]; // Noise
    buffer.extend_from_slice(&frame);
    let parser = Parser::new();

    let (msg, consumed) = parser.parse(&buffer).unwrap().unwrap();
    assert_eq!(consumed, buffer.len());
    match msg {
        klipper_proto::commands::Message::Command(Command::GetStatus) => { /* Success */ }
        _ => panic!("Incorrect message parsed"),
    }
}

#[test]
fn test_parse_multiple_messages() {
    let frame1 = build_test_frame(4, 0x02, &[]); // GetConfig
    let frame2 = build_test_frame(5, 0x03, &[]); // GetStatus
    let mut buffer = Vec::new();
    buffer.extend_from_slice(&frame1);
    buffer.extend_from_slice(&frame2);

    let parser = Parser::new();

    // First message
    let (msg1, consumed1) = parser.parse(&buffer).unwrap().unwrap();
    assert_eq!(consumed1, frame1.len());
    assert!(matches!(
        msg1,
        klipper_proto::commands::Message::Command(Command::GetConfig)
    ));

    // Second message
    let (msg2, consumed2) = parser.parse(&buffer[consumed1..]).unwrap().unwrap();
    assert_eq!(consumed2, frame2.len());
    assert!(matches!(
        msg2,
        klipper_proto::commands::Message::Command(Command::GetStatus)
    ));
}

#[test]
fn test_incomplete_frame_returns_none() {
    let frame = build_test_frame(6, 0x02, &[]);
    let parser = Parser::new();

    // Frame is missing the last byte
    let result = parser.parse(&frame[..frame.len() - 1]);
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
}

#[test]
fn test_bad_crc_returns_err() {
    let mut frame = build_test_frame(7, 0x02, &[]);
    let last_byte_idx = frame.len() - 1;
    frame[last_byte_idx] ^= 0xFF; // Corrupt CRC

    let parser = Parser::new();
    let result = parser.parse(&frame);
    assert!(result.is_err());
    let (err, consumed) = result.unwrap_err();
    assert_eq!(err, klipper_proto::Error::InvalidCrc);
    assert_eq!(consumed, 1); // Should discard the bad sync byte and try again
}

#[test]
fn test_no_sync_byte_returns_err() {
    let buffer = vec![0x00, 0x01, 0x02, 0x03, 0x04];
    let parser = Parser::new();
    let result = parser.parse(&buffer);
    assert!(result.is_err());
    let (err, consumed) = result.unwrap_err();
    assert_eq!(err, klipper_proto::Error::InvalidSync);
    assert_eq!(consumed, buffer.len()); // Should discard the whole buffer
}
