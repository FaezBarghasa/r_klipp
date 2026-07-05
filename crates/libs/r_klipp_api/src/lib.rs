#![cfg_attr(not(feature = "std"), no_std)]

pub mod hal;

use heapless;
use serde::{Serialize, Deserialize};
use fixed::types::I32F32;

pub type FixedPoint = I32F32;

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TrajectoryCoefficients {
    pub points: heapless::Vec<[FixedPoint; 3], 8>,
    pub time_params: TimeParams,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TimeParams {
    pub start_time: u64,
    pub duration: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct McuConfig {
    pub microsteps: u8,
    pub current: u16,
    pub resonance_freq: u32,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum FaultCode {
    ThermalRunaway,
    StallGuard,
    EmergencyStop,
    UnknownCommand,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum HostToMcu {
    Trajectory(TrajectoryCoefficients),
    EmergencyStop,
    ConfigUpdate(McuConfig),
    SyncClock(u64),
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub enum McuToHost {
    Telemetry {
        pos: [FixedPoint; 6],
        temps: [FixedPoint; 4],
        currents: [FixedPoint; 8],
    },
    Ack,
    Fault(FaultCode),
}

#[cfg(test)]
mod tests {
    use super::*;
    use postcard::{to_vec, from_bytes};

    #[test]
    fn test_host_to_mcu_serialization() {
        let mut points = heapless::Vec::new();
        points.push([FixedPoint::from_num(1.0), FixedPoint::from_num(2.0), FixedPoint::from_num(3.0)]).unwrap();
        points.push([FixedPoint::from_num(4.0), FixedPoint::from_num(5.0), FixedPoint::from_num(6.0)]).unwrap();

        let trajectory = HostToMcu::Trajectory(TrajectoryCoefficients {
            points,
            time_params: TimeParams { start_time: 12345, duration: 100 },
        });

        let serialized = to_vec::<_, 256>(&trajectory).unwrap();
        let deserialized: HostToMcu = from_bytes(&serialized).unwrap();

        assert_eq!(trajectory, deserialized);
    }

    #[test]
    fn test_mcu_to_host_serialization() {
        let telemetry = McuToHost::Telemetry {
            pos: [
                FixedPoint::from_num(1.0), FixedPoint::from_num(2.0), FixedPoint::from_num(3.0),
                FixedPoint::from_num(4.0), FixedPoint::from_num(5.0), FixedPoint::from_num(6.0)
            ],
            temps: [
                FixedPoint::from_num(10.0), FixedPoint::from_num(20.0),
                FixedPoint::from_num(30.0), FixedPoint::from_num(40.0)
            ],
            currents: [
                FixedPoint::from_num(0.1), FixedPoint::from_num(0.2), FixedPoint::from_num(0.3),
                FixedPoint::from_num(0.4), FixedPoint::from_num(0.5), FixedPoint::from_num(0.6),
                FixedPoint::from_num(0.7), FixedPoint::from_num(0.8)
            ],
        };

        let serialized = to_vec::<_, 256>(&telemetry).unwrap();
        let deserialized: McuToHost = from_bytes(&serialized).unwrap();

        assert_eq!(telemetry, deserialized);
    }
}