//! Integration tests for klipper-proto codec, clock sync, and connection lifecycle.

use klipper_proto::codec::{StepEncoder, StepPacket};
use klipper_proto::clock_sync::DpllClockSync;
use klipper_proto::connection::{ConnectionManager, ConnectionState};
use klipper_proto::registry::CommandRegistry;

#[test]
fn test_step_packet_encode_validate() {
    let pkt = StepPacket::new(100_000, 250, 1, 0);
    assert!(pkt.validate());
    assert_eq!(pkt.timestamp_us, 100_000);
    assert_eq!(pkt.interval_us, 250);
}

#[test]
fn test_step_encoder_queue() {
    let mut enc = StepEncoder::new();
    assert!(enc.encode(1_000, 0, 0).is_ok());
    assert!(enc.encode(1_200, 0, 0).is_ok());

    let p1 = enc.try_dequeue().unwrap();
    let p2 = enc.try_dequeue().unwrap();
    assert_eq!(p1.timestamp_us, 1_000);
    assert_eq!(p2.interval_us, 200);
}

#[test]
fn test_dpll_clock_synchronizer() {
    let mut dpll = DpllClockSync::new();
    for i in 0..10 {
        let send = 1_000_000 + i * 100_000;
        let recv = send + 150;
        let mcu_ticks = (send + 75) * 84; // 84MHz timer
        assert!(dpll.add_sample(send, mcu_ticks, recv));
    }

    let query_host_us = 2_500_000;
    let ticks = dpll.host_to_mcu_ticks(query_host_us);
    let back_us = dpll.mcu_to_host_us(ticks);
    assert!((back_us as i64 - query_host_us as i64).abs() <= 2);
}

#[test]
fn test_command_registry() {
    let mut reg = CommandRegistry::new();
    reg.add("get_config", 1);
    reg.add("queue_step", 2);
    reg.add("emergency_stop", 3);

    assert_eq!(reg.get_id("get_config"), Some(1));
    assert_eq!(reg.get_id("queue_step"), Some(2));
    assert_eq!(reg.get_name(3), Some("emergency_stop"));
    assert_eq!(reg.get_id("unknown"), None);
}

#[test]
fn test_connection_state_machine() {
    let mut conn = ConnectionManager::new(250, 1000);
    conn.on_connected(500);
    assert_eq!(conn.state, ConnectionState::Connected);

    conn.on_heartbeat_received(1200);
    assert!(conn.check_health(1400).is_ok());

    assert!(conn.check_health(2500).is_err());
    assert!(matches!(conn.state, ConnectionState::Reconnecting { attempt: 1 }));
}
