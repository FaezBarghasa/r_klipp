//! Unit tests and integration tests for the Klipper protocol implementation.

#[cfg(feature = "std")]
mod std_tests {
    use klipper_proto::codec::KlipperCodec;
    use klipper_proto::commands::{Command, Message, Response};
    use bytes::BytesMut;
    use tokio_util::codec::{Encoder, Decoder};

    /// Helper function to test a full encode -> decode roundtrip.
    fn roundtrip(message: Message) {
        let mut codec = KlipperCodec::new();
        let mut buffer = BytesMut::new();

        // Encode the message
        codec.encode(message.clone(), &mut buffer).unwrap();

        // Decode it back
        let decoded = codec.decode(&mut buffer).unwrap().unwrap();

        // Assert they are identical
        assert_eq!(message, decoded);

        // Assert the buffer is fully consumed
        assert!(buffer.is_empty(), "Buffer should be empty after decoding a full frame");
    }

    #[test]
    fn roundtrip_get_config() {
        roundtrip(Message::Command(Command::GetConfig));
    }

    #[test]
    fn roundtrip_response_config() {
        roundtrip(Message::Response(Response::Config {
            is_config_valid: true,
            mcu_version: 12345,
            mcu_name: "test_mcu".into(),
        }));
    }

    #[test]
    fn roundtrip_gcode_command() {
        roundtrip(Message::Command(Command::GCode("G1 X10 Y10 F3000".into())));
    }

    #[test]
    fn roundtrip_gcode_ok() {
        roundtrip(Message::Response(Response::GCodeOk));
    }

    #[test]
    fn roundtrip_gcode_error() {
        roundtrip(Message::Response(Response::GCodeError("Unknown command".into())));
    }

    #[test]
    fn roundtrip_message_with_escapable_bytes() {
        // Create a message containing bytes that need escaping (0x7E, 0x7D)
        roundtrip(Message::Command(Command::GCode("G1 X~} F3000".replace("~", "\x7E").replace("}", "\x7D").into())));
    }

    #[test]
    fn decode_incomplete_frame_returns_none() {
        let mut codec = KlipperCodec::new();
        // A valid header but not enough payload data
        let mut buffer = BytesMut::from(&[0x7E, 0x0A, 0x00, 0x01, 0x02, 0x03][..]);

        let result = codec.decode(&mut buffer).unwrap();
        assert!(result.is_none());
    }

    #[test]
    fn decode_bad_crc_returns_err() {
        let mut codec = KlipperCodec::new();
        let mut buffer = BytesMut::new();
        let message = Message::Command(Command::GetConfig);

        codec.encode(message, &mut buffer).unwrap();

        // Corrupt the CRC byte (index 2)
        if buffer.len() > 2 {
            buffer[2] = buffer[2].wrapping_add(1);
        }

        let result = codec.decode(&mut buffer);
        assert!(result.is_err());
    }

    #[test]
    fn decode_stream_with_leading_noise() {
        let mut codec = KlipperCodec::new();
        let mut buffer = BytesMut::new();
        let message = Message::Command(Command::EmergencyStop);

        // Add some garbage bytes at the beginning
        buffer.extend_from_slice(&[0xDE, 0xAD, 0xBE, 0xEF]);

        // Add a valid frame after the noise
        codec.encode(message.clone(), &mut buffer).unwrap();

        // The decoder should skip the noise and find the frame
        let decoded = codec.decode(&mut buffer).unwrap().unwrap();
        assert_eq!(message, decoded);
    }
}

