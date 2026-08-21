//! Packet byte buffer utilities for Host-MCU transport.

use heapless::Vec;

pub struct PacketBuffer<const N: usize> {
    pub buffer: Vec<u8, N>,
}

impl<const N: usize> PacketBuffer<N> {
    pub fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    pub fn write_bytes(&mut self, data: &[u8]) -> Result<(), ()> {
        self.buffer.extend_from_slice(data).map_err(|_| ())
    }

    pub fn clear(&mut self) {
        self.buffer.clear();
    }

    pub fn as_slice(&self) -> &[u8] {
        self.buffer.as_slice()
    }
}
