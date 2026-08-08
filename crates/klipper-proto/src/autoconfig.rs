use serde::{Deserialize, Serialize};
use alloc::string::String;
use alloc::vec::Vec;
use alloc::string::ToString;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(default)]
#[non_exhaustive]
pub struct BoardManifest {
    pub board_name: String,
    pub mcu_uid: [u8; 12],
    pub step_timer_hz: u32,
    pub step_drivers: Vec<DriverPinout>,
    pub temperature_adc_channels: Vec<AdcPinout>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct DriverPinout {
    pub label: String,
    pub step_pin: u16,
    pub dir_pin: u16,
    pub enable_pin: u16,
    pub spi_bus_id: Option<u8>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[non_exhaustive]
pub struct AdcPinout {
    pub label: String,
    pub adc_pin: u16,
    pub channel: u8,
}

impl Default for BoardManifest {
    fn default() -> Self {
        Self {
            board_name: "Unknown".to_string(),
            mcu_uid: [0; 12],
            step_timer_hz: 0,
            step_drivers: Vec::new(),
            temperature_adc_channels: Vec::new(),
        }
    }
}