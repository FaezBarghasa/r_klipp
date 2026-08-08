use crate::hal::traits::Can;

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Cia402State {
    SwitchOnDisabled,
    ReadyToSwitchOn,
    SwitchedOn,
    OperationEnabled,
    // ... other states
}

pub struct Cia402Driver<C: Can> {
    can: C,
    state: Cia402State,
}

impl<C: Can> Cia402Driver<C> {
    pub async fn transition_to(&mut self, target_state: Cia402State) -> Result<(), ()> {
        // Implementation of the CiA 402 state machine transitions
        self.state = target_state;
        Ok(())
    }
}

#[embassy_executor::task]
pub async fn cia402_proxy_task(
    mut driver: Cia402Driver<impl Can + 'static>,
    // Add command channel
) {
    loop {
        // Receive position commands and format them as CSP frames
        // Send frames via the CAN bus
        embassy_time::Timer::after_millis(1).await;
    }
}
