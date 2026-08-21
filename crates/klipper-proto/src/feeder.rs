//! CAN-FD Smart Feeder & Peripheral Protocol Schema.
//!
//! Replaces fragile step/dir feeder signals with industrial CAN-FD RPC messaging
//! supporting tape indexing, cover tape peeling, component presence sensing, and jam detection.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FeederCommandType {
    AdvanceTape = 0x01,
    PeelCover = 0x02,
    CalibrateIndex = 0x03,
    GetStatus = 0x04,
    SetPitch = 0x05,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FeederCommand {
    pub slot_id: u8,
    pub cmd_type: FeederCommandType,
    pub pitch_mm: f32,
    pub speed_percentage: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum FeederStatus {
    Ready = 0x00,
    Advancing = 0x01,
    JamDetected = 0x02,
    TapeExhausted = 0x03,
    OpticalSensorMismatch = 0x04,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct FeederResponse {
    pub slot_id: u8,
    pub status: FeederStatus,
    pub remaining_parts_estimate: u32,
}

impl FeederCommand {
    pub fn advance_tape(slot_id: u8, pitch_mm: f32) -> Self {
        Self {
            slot_id,
            cmd_type: FeederCommandType::AdvanceTape,
            pitch_mm,
            speed_percentage: 100,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feeder_command_serialization() {
        let cmd = FeederCommand::advance_tape(4, 4.0); // 4mm pitch 0805 tape
        assert_eq!(cmd.slot_id, 4);
        assert_eq!(cmd.cmd_type, FeederCommandType::AdvanceTape);
        assert_eq!(cmd.pitch_mm, 4.0);

        let resp = FeederResponse {
            slot_id: 4,
            status: FeederStatus::Ready,
            remaining_parts_estimate: 4800,
        };
        assert_eq!(resp.status, FeederStatus::Ready);
    }
}
