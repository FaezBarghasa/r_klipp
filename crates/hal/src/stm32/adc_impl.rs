use crate::traits::AdcDma;

pub struct Stm32AdcDma;

impl AdcDma for Stm32AdcDma {
    type Error = core::convert::Infallible;
    fn start_streaming(&mut self, _buffer: &mut [u16]) -> Result<(), Self::Error> {
        Ok(())
    }
}

