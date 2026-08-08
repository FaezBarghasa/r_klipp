use hal::spi::SpiDevice;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct VirtualSpi {
    // We can add more sophisticated simulation logic here if needed,
    // like a shared buffer or a mock device.
}

impl VirtualSpi {
    pub fn new() -> Self {
        Self {}
    }
}

impl SpiDevice for VirtualSpi {
    type Error = ();

    async fn write(&mut self, _bytes: &[u8]) -> Result<(), Self::Error> {
        // In a real simulation, we would send this to a mock device.
        Ok(())
    }

    async fn read(&mut self, bytes: &mut [u8]) -> Result<(), Self::Error> {
        // In a real simulation, we would receive this from a mock device.
        bytes.fill(0);
        Ok(())
    }

    async fn transfer(&mut self, read: &mut [u8], _write: &[u8]) -> Result<(), Self::Error> {
        read.fill(0);
        Ok(())
    }

    async fn transfer_in_place(&mut self, bytes: &mut [u8]) -> Result<(), Self::Error> {
        bytes.fill(0);
        Ok(())
    }
}
