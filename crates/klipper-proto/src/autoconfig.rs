#![no_std]

use heapless::Vec;
use serde::{Deserialize, Serialize};

/// Defines the hardware capabilities of a specific MCU pin.
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum PinCapability {
    DigitalInput,
    DigitalOutput { max_current_ma: u8 },
    AnalogInput { resolution_bits: u8 },
    PwmOutput { max_freq_hz: u32 },
    StepTimerChannel { timer_id: u8 },
}

/// Describes a physical pin on the MCU and its available capabilities.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PinDescriptor {
    /// The hardware index of the pin.
    pub pin_index: u16,
    /// A fixed-length null-padded string representing the pin name.
    pub name: [u8; 8],
    /// A bitmask of supported alternate functions and capabilities.
    pub capabilities_bitmask: u32,
}

/// The main payload structure for handshake configuration.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HandshakeManifest {
    /// A fixed-length null-padded string representing the board's name.
    pub board_name: [u8; 32],
    /// The core clock frequency of the MCU in Hertz.
    pub clock_speed_hz: u32,
    /// The timer resolution in ticks used for step generation.
    pub step_resolution_ticks: u32,
    /// A statically sized vector of pin descriptors.
    pub pins: Vec<PinDescriptor, 64>,
}

impl HandshakeManifest {
    /// Serializes the manifest into a provided byte buffer using postcard.
    ///
    /// Returns a slice of the buffer containing the serialized data on success,
    /// or a static string error message on failure.
    pub fn serialize_to_buffer<'a>(&self, buffer: &'a mut [u8]) -> Result<&'a mut [u8], &'static str> {
        postcard::to_slice(self, buffer).map_err(|_| "Failed to serialize HandshakeManifest")
    }

    /// Deserializes a manifest from a byte slice using postcard.
    ///
    /// Returns a new HandshakeManifest instance on success, or a static
    /// string error message on failure.
    pub fn deserialize_from_slice(slice: &[u8]) -> Result<Self, &'static str> {
        postcard::from_bytes(slice).map_err(|_| "Failed to deserialize HandshakeManifest")
    }
}
