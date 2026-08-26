use klipper_proto::codec::{StepEncoder, StepPacket};
use klipper_proto::registry::CommandRegistry;

#[test]
fn test_step_packet_crc_corruption_detected() {
    let mut pkt = StepPacket::new(100_000, 50, 1, 0);
    assert!(pkt.validate(), "Valid packet should pass validation");

    // Corrupt packet timestamp
    pkt.timestamp_us ^= 0x01;
    assert!(!pkt.validate(), "Corrupted packet should fail CRC validation");
}

#[test]
fn test_step_packet_direction_change() {
    let forward_pkt = StepPacket::new(50_000, 20, 2, 0);
    let reverse_pkt = StepPacket::new(50_000, 20, 2, 1);

    assert!(forward_pkt.validate());
    assert!(reverse_pkt.validate());
    assert_ne!(forward_pkt.crc16, reverse_pkt.crc16, "Different directions must have different CRCs");
}

#[test]
fn test_step_encoder_overflow_returns_err() {
    let mut encoder = StepEncoder::new();

    let mut count = 0;
    while encoder.encode(1000 + count * 10, 0, 0).is_ok() {
        count += 1;
    }
    assert!(count >= 1000, "Should enqueue at least 1000 items before filling, got {}", count);

    // Subsequent packet must fail with Err(())
    assert!(encoder.encode(20_000, 0, 0).is_err(), "Must return Err on full queue");

    // Dequeue one packet, then enqueueing should succeed again
    assert!(encoder.try_dequeue().is_some());
    assert!(encoder.encode(20_001, 0, 0).is_ok(), "Enqueue should succeed after dequeue");

}

#[test]
fn test_step_encoder_interval_wrapping() {
    let mut encoder = StepEncoder::new();

    // Large timestamp close to u32 max
    assert!(encoder.encode(0xFFFF_FF00, 1, 0).is_ok());
    // Wrapped timestamp
    assert!(encoder.encode(0x0000_0100, 1, 0).is_ok());

    let _p1 = encoder.try_dequeue().unwrap();
    let p2 = encoder.try_dequeue().unwrap();

    // (0x0000_0100 wrapping_sub 0xFFFF_FF00) as u16 = 0x200 (512)
    assert_eq!(p2.interval_us, 512, "Wrapped timestamp interval should be calculated correctly");
}

#[test]
fn test_command_registry_bidirectional() {
    let mut registry = CommandRegistry::new();
    registry.add("emergency_stop", 1);
    registry.add("step_stepper", 2);
    registry.add("set_pwm", 3);

    assert_eq!(registry.get_id("emergency_stop"), Some(1));
    assert_eq!(registry.get_id("step_stepper"), Some(2));
    assert_eq!(registry.get_id("set_pwm"), Some(3));
    assert_eq!(registry.get_id("unknown_command"), None);

    assert_eq!(registry.get_name(1), Some("emergency_stop"));
    assert_eq!(registry.get_name(2), Some("step_stepper"));
    assert_eq!(registry.get_name(3), Some("set_pwm"));
    assert_eq!(registry.get_name(99), None);
}
