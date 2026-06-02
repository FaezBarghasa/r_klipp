// crates/klipper-proto/src/autoconfig.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum PinCapability {
    DigitalInput = 1,
    DigitalOutput = 2,
    AnalogInput = 4,
    PwmOutput = 8,
    StepTimerChannel = 16,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct PinDescriptor {
    pub pin_index: u16,        // PA3 -> 3, PB11 -> 27
    pub name: [u8; 8],         // UTF-8 Fixed-length name (null-padded)
    pub capabilities: u16,     // Bitmask of supported PinCapability values
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HandshakeManifest {
    pub board_name: [u8; 32],
    pub clock_speed_hz: u32,
    pub step_resolution_ticks: u32,
    pub pins: heapless::Vec<PinDescriptor, 64>,
}

impl HandshakeManifest {
    /// Serializes the handshake manifest into a byte buffer using postcard for no_std environments
    pub fn serialize_to_buffer<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a [u8], &'static str> {
        postcard::to_slice(self, buffer)
            .map(|s| s as &[u8])
            .map_err(|_| "Postcard serialization failed")
    }

    /// Deserializes a manifest received from an MCU
    pub fn deserialize_from_slice(slice: &[u8]) -> Result<Self, &'static str> {
        postcard::from_bytes(slice).map_err(|_| "Postcard deserialization failed")
    }
}
