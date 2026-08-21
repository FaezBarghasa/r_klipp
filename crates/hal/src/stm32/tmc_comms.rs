use crate::traits::HighSpeedSpi;

pub struct Stm32HighSpeedSpi;

impl HighSpeedSpi for Stm32HighSpeedSpi {
    type Error = core::convert::Infallible;
    fn transfer(&mut self, _write: &[u8], _read: &mut [u8]) -> Result<(), Self::Error> {
        Ok(())
    }
}


// Implementation for single-wire UART will be added here.
