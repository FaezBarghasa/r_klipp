pub enum PinState {
    Low,
    High,
}

pub trait InputPin {
    type Error;

    fn is_high(&self) -> Result<bool, Self::Error>;
    fn is_low(&self) -> Result<bool, Self::Error>;
}

pub trait OutputPin {
    type Error;

    fn set_high(&mut self) -> Result<(), Self::Error>;
    fn set_low(&mut self) -> Result<(), Self::Error>;
}

pub trait StatefulOutputPin: OutputPin {
    fn set_state(&mut self, state: PinState) -> Result<(), Self::Error>;
}

pub trait ToggleableOutputPin: OutputPin {
    fn toggle(&mut self) -> Result<(), Self::Error>;
}
