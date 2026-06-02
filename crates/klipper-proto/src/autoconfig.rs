// crates/klipper-proto/src/autoconfig.rs
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinCapability {
    DigitalInput,
    DigitalOutput { max_current_ma: u8 },
    AnalogInput { resolution_bits: u8 },
    PwmOutput { max_freq_hz: u32 },
    StepTimerChannel { timer_id: u8 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PinDescriptor {
    pub pin_index: u16,        // PA3 -> 3, PB11 -> 27
    pub name: [u8; 8],         // UTF-8 Fixed-length name (null-padded)
    pub capabilities_mask: u16, // Bitmask of supported PinCapability flags
    pub capabilities: heapless::Vec<PinCapability, 4>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HandshakeManifest {
    pub board_name: [u8; 32],
    pub clock_speed_hz: u32,
    pub step_resolution_ticks: u32,
    pub pins: heapless::Vec<PinDescriptor, 64>,
}

impl HandshakeManifest {
    /// Creates a new HandshakeManifest with the specified board name and clock parameters.
    pub fn new(board: &str, clock_speed_hz: u32, step_resolution_ticks: u32) -> Self {
        let mut board_name = [0u8; 32];
        let bytes = board.as_bytes();
        let len = bytes.len().min(32);
        board_name[..len].copy_from_slice(&bytes[..len]);
        Self {
            board_name,
            clock_speed_hz,
            step_resolution_ticks,
            pins: heapless::Vec::new(),
        }
    }

    /// Registers a pin with its capabilities to the manifest.
    pub fn add_pin(
        &mut self, 
        pin_index: u16, 
        name: &str, 
        capabilities_mask: u16,
        capabilities: heapless::Vec<PinCapability, 4>
    ) -> Result<(), &'static str> {
        let mut pin_name = [0u8; 8];
        let bytes = name.as_bytes();
        let len = bytes.len().min(8);
        pin_name[..len].copy_from_slice(&bytes[..len]);
        
        self.pins.push(PinDescriptor {
            pin_index,
            name: pin_name,
            capabilities_mask,
            capabilities,
        }).map_err(|_| "Pin capacity exceeded (max 64)")
    }

    /// Finds a pin descriptor by its name.
    pub fn find_pin_by_name(&self, name: &str) -> Option<&PinDescriptor> {
        let mut search_name = [0u8; 8];
        let bytes = name.as_bytes();
        let len = bytes.len().min(8);
        search_name[..len].copy_from_slice(&bytes[..len]);
        
        self.pins.iter().find(|p| p.name == search_name)
    }

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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_handshake_manifest_serialization() {
        let mut manifest = HandshakeManifest::new("MKS SKIPR", 168_000_000, 1000);
        
        let mut caps1 = heapless::Vec::new();
        caps1.push(PinCapability::DigitalInput).unwrap();
        caps1.push(PinCapability::DigitalOutput { max_current_ma: 20 }).unwrap();
        manifest.add_pin(3, "PA3", 3, caps1).unwrap();
        
        let mut caps2 = heapless::Vec::new();
        caps2.push(PinCapability::PwmOutput { max_freq_hz: 10_000 }).unwrap();
        manifest.add_pin(27, "PB11", 8, caps2).unwrap();

        assert_eq!(manifest.pins.len(), 2);
        assert!(manifest.find_pin_by_name("PA3").is_some());
        assert_eq!(manifest.find_pin_by_name("PA3").unwrap().pin_index, 3);

        let mut buffer = [0u8; 512];
        let serialized = manifest.serialize_to_buffer(&mut buffer).unwrap();
        assert!(serialized.len() > 0);

        let deserialized = HandshakeManifest::deserialize_from_slice(serialized).unwrap();
        assert_eq!(deserialized.clock_speed_hz, 168_000_000);
        assert_eq!(deserialized.pins.len(), 2);
        assert_eq!(deserialized.find_pin_by_name("PB11").unwrap().pin_index, 27);
    }
}
