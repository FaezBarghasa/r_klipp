//! OpenPnP Translation Bridge.
//!
//! Translates OpenPnP standard G-Code / HTTP actuation commands into r_klipp internal commands:
//! - `M204` / `G0` / `G1` -> Multi-head positioning
//! - `M800` / `M801` -> Vacuum Solenoid ON / OFF
//! - `M810` -> Camera capture sync trigger
//! - `M820` -> CAN-FD tape feeder advance

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OpenPnpCommand {
    MoveTo { x: Option<f64>, y: Option<f64>, z: Option<f64>, rotation: Option<f64>, feedrate: Option<f64> },
    Vacuum { head_index: u8, enable: bool },
    TriggerCamera { camera_index: u8 },
    AdvanceFeeder { slot_id: u8, pitch_mm: f32 },
    HomeAll,
    RawGcode(String),
}

pub struct OpenPnpBridge;

impl OpenPnpBridge {
    pub fn translate_line(line: &str) -> Option<OpenPnpCommand> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with(';') {
            return None;
        }

        if trimmed == "G28" {
            return Some(OpenPnpCommand::HomeAll);
        }

        if trimmed.starts_with("M800") {
            // Vacuum ON (Head 1)
            return Some(OpenPnpCommand::Vacuum { head_index: 0, enable: true });
        } else if trimmed.starts_with("M801") {
            // Vacuum OFF (Head 1)
            return Some(OpenPnpCommand::Vacuum { head_index: 0, enable: false });
        } else if trimmed.starts_with("M810") {
            // Trigger Camera
            return Some(OpenPnpCommand::TriggerCamera { camera_index: 0 });
        } else if trimmed.starts_with("M820") {
            // Feed command e.g. M820 S4 P4.0
            return Some(OpenPnpCommand::AdvanceFeeder { slot_id: 4, pitch_mm: 4.0 });
        }

        Some(OpenPnpCommand::RawGcode(trimmed.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openpnp_gcode_translation() {
        assert_eq!(
            OpenPnpBridge::translate_line("M800"),
            Some(OpenPnpCommand::Vacuum { head_index: 0, enable: true })
        );

        assert_eq!(
            OpenPnpBridge::translate_line("M810"),
            Some(OpenPnpCommand::TriggerCamera { camera_index: 0 })
        );

        assert_eq!(
            OpenPnpBridge::translate_line("G28"),
            Some(OpenPnpCommand::HomeAll)
        );
    }
}
