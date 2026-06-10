#![no_std]

use heapless::Vec;
use serde::{Deserialize, Serialize};

/// Enumerates the hardware capabilities available on a microcontroller pin.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PinCapability {
    DigitalInput,
    DigitalOutput { max_current_ma: u8 },
    AnalogInput { resolution_bits: u8 },
    PwmOutput { max_freq_hz: u32 },
    StepTimerChannel { timer_id: u8 },
}

/// Describes an individual pin and its assigned hardware capabilities.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct PinDescriptor {
    pub pin_index: u16,
    pub name: [u8; 8],
    pub capabilities: u32, // Bitmask mapping to PinCapability options
}

/// The Handshake Manifest sent from MCU to Host to configure the workspace layout.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandshakeManifest {
    pub board_name: [u8; 32],
    pub clock_speed_hz: u32,
    pub step_resolution_ticks: u32,
    pub pins: Vec<PinDescriptor, 64>,
}

impl HandshakeManifest {
    /// Serializes the manifest into a binary postcard format buffer.
    pub fn serialize_to_buffer<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a [u8], &'static str> {
        postcard::to_slice(self, buffer).map_err(|_| "Failed to serialize handshake manifest")
    }

    /// Deserializes a postcard binary slice into a configuration manifest struct.
    pub fn deserialize_from_slice(slice: &[u8]) -> Result<Self, &'static str> {
        postcard::from_bytes(slice).map_err(|_| "Failed to deserialize handshake manifest")
    }
}