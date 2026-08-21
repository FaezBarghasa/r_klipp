#![cfg_attr(not(feature = "std"), no_std)]

pub use postcard;
use heapless::Vec;
use serde::{Deserialize, Serialize};

/// Current protocol schema version for host <-> MCU communication.
pub const CURRENT_SCHEMA_VERSION: u32 = 1;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FixedPoint {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct Waypoint {
    pub position: [f32; 3],
    pub feedrate: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum HostCommand {
    PredictiveTrajectory {
        nurbs_points: Vec<FixedPoint, 16>,
        mpcc_feedforward: [f32; 3],
    },
    BasicTrajectory {
        waypoints: Vec<Waypoint, 32>,
        max_jerk: f32,
    },
    StepMove {
        interval_ticks: u32,
        step_mask: u8,
        dir_mask: u8,
    },
    SetPwm {
        channel: u8,
        duty: f32,
    },
    SyncClock {
        host_timestamp_us: u64,
    },
    EmergencyStop,
    GetStatus,
}

/// Versioned Host-to-MCU message envelope.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct HostToMcu {
    pub schema_version: u32,
    pub sequence_id: u32,
    pub command: HostCommand,
}

impl HostToMcu {
    pub fn new(sequence_id: u32, command: HostCommand) -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            sequence_id,
            command,
        }
    }

    /// Verifies that the message schema version matches the MCU expected version.
    pub fn validate_version(&self) -> Result<(), u32> {
        if self.schema_version == CURRENT_SCHEMA_VERSION {
            Ok(())
        } else {
            Err(self.schema_version)
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct LinkHealth {
    pub rtt_us: u32,
    pub buffer_fill_percent: u8,
    pub dropped_packets: u16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaultCode {
    Unknown,
    ThermalRunaway,
    BufferStarvation,
    GcodeError,
    VersionMismatch { expected: u32, received: u32 },
    WatchdogTimeout,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum McuPayload {
    Telemetry {
        pos: [f32; 6],
        temps: [f32; 4],
        link_health: LinkHealth,
    },
    Ack { sequence_id: u32 },
    ClockSyncReply { host_time_us: u64, mcu_time_ticks: u64 },
    Fault(FaultCode),
}

/// Versioned MCU-to-Host message envelope.
#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct McuToHost {
    pub schema_version: u32,
    pub payload: McuPayload,
}

impl McuToHost {
    pub fn new(payload: McuPayload) -> Self {
        Self {
            schema_version: CURRENT_SCHEMA_VERSION,
            payload,
        }
    }
}

pub mod hal {
    // Protocol hal markers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_versioned_postcard_roundtrip() {
        let msg = HostToMcu::new(42, HostCommand::SyncClock { host_timestamp_us: 12345678 });
        let mut buf = [0u8; 128];
        let serialized = postcard::to_slice(&msg, &mut buf).unwrap();

        let deserialized: HostToMcu = postcard::from_bytes(serialized).unwrap();
        assert_eq!(deserialized.schema_version, CURRENT_SCHEMA_VERSION);
        assert_eq!(deserialized.sequence_id, 42);
        assert!(deserialized.validate_version().is_ok());
    }

    #[test]
    fn test_version_mismatch_rejection() {
        let mut msg = HostToMcu::new(1, HostCommand::EmergencyStop);
        msg.schema_version = 99; // Mismatched version
        assert_eq!(msg.validate_version(), Err(99));
    }
}
