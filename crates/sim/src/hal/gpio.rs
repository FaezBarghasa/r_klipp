use hal::gpio::{InputPin, OutputPin, PinState, StatefulOutputPin, ToggleableOutputPin};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct VirtualPin {
    state: Arc<Mutex<PinState>>,
}

impl VirtualPin {
    pub fn new(initial_state: PinState) -> Self {
        Self {
            state: Arc::new(Mutex::new(initial_state)),
        }
    }
}

impl InputPin for VirtualPin {
    type Error = ();

    fn is_high(&self) -> Result<bool, Self::Error> {
        Ok(*self.state.lock().unwrap() == PinState::High)
    }

    fn is_low(&self) -> Result<bool, Self::Error> {
        Ok(*self.state.lock().unwrap() == PinState::Low)
    }
}

impl OutputPin for VirtualPin {
    type Error = ();

    fn set_high(&mut self) -> Result<(), Self::Error> {
        *self.state.lock().unwrap() = PinState::High;
        Ok(())
    }

    fn set_low(&mut self) -> Result<(), Self::Error> {
        *self.state.lock().unwrap() = PinState::Low;
        Ok(())
    }
}

impl StatefulOutputPin for VirtualPin {
    fn set_state(&mut self, state: PinState) -> Result<(), Self::Error> {
        *self.state.lock().unwrap() = state;
        Ok(())
    }
}

impl ToggleableOutputPin for VirtualPin {
    fn toggle(&mut self) -> Result<(), Self::Error> {
        let mut state = self.state.lock().unwrap();
        *state = match *state {
            PinState::High => PinState::Low,
            PinState::Low => PinState::High,
        };
        Ok(())
    }
}
