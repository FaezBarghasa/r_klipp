#![cfg_attr(not(feature = "std"), no_std)]

use heapless;
use postcard;

// Re-export postcard for convenience
pub use postcard;

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct FixedPoint {
    pub x: i32,
    pub y: i32,
    pub z: i32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct Waypoint {
    pub position: [f32; 3],
    pub feedrate: f32,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum HostToMcu {
    PredictiveTrajectory {
        nurbs_points: heapless::Vec<FixedPoint, 16>,
        mpcc_feedforward: [f32; 3],
    },
    BasicTrajectory {
        waypoints: heapless::Vec<Waypoint, 32>,
        max_jerk: f32,
    },
    EmergencyStop,
    SyncClock(u64),
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub struct LinkHealth {
    pub rtt_us: u32,
    pub buffer_fill_percent: u8,
    pub dropped_packets: u16,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum FaultCode {
    Unknown,
    ThermalRunaway,
    BufferStarvation,
    GcodeError,
}

#[derive(serde::Serialize, serde::Deserialize, Debug, PartialEq)]
pub enum McuToHost {
    Telemetry {
        pos: [f32; 6],
        temps: [f32; 4],
        link_health: LinkHealth,
    },
    Ack,
    Fault(FaultCode),
}

pub mod hal {
    // This will be filled in Task 0.3
}
