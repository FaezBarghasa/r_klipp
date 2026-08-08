use crate::gpio::{InputPin, OutputPin, PinState, StatefulOutputPin, ToggleableOutputPin};
use embassy_stm32::gpio::{AnyPin, Input, Output, Level, Speed};

pub struct Stm32InputPin<'d> {
    pin: Input<'d, AnyPin>,
}

impl<'d> Stm32InputPin<'d> {
    pub fn new(pin: AnyPin) -> Self {
        Self {
            pin: Input::new(pin, embassy_stm32::gpio::Pull::None),
        }
    }
}

impl<'d> InputPin for Stm32InputPin<'d> {
    type Error = ();

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(self.pin.is_high())
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(self.pin.is_low())
    }
}

pub struct Stm32OutputPin<'d> {
    pin: Output<'d, AnyPin>,
}

impl<'d> Stm32OutputPin<'d> {
    pub fn new(pin: AnyPin) -> Self {
        Self {
            pin: Output::new(pin, Level::Low, Speed::Low),
        }
    }
}

impl<'d> OutputPin for Stm32OutputPin<'d> {
    type Error = ();

    fn set_high(&mut self) -> Result<(), Self::Error> {
        self.pin.set_high();
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        self.pin.set_low();
        Ok(())
    }
}

impl<'d> StatefulOutputPin for Stm32OutputPin<'d> {
    fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> {
        match state {
            PinState::High => self.pin.set_high(),
            PinState::Low => self.pin.set_low(),
        }
        Ok(())
    }
}

impl<'d> ToggleableOutputPin for Stm32OutputPin<'d> {
    fn toggle(&mut self) -> Result<(), Self::Error> {
        self.pin.toggle();
        Ok(())
    }
}
