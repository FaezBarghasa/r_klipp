//! A `no_std` driver for the DRV8825 stepper motor driver.
//!
//! This driver controls the DRV8825 using GPIO pins for step, direction,
//! enable, and microstep configuration.

#![no_std]

use embedded_hal::digital::{OutputPin, PinState};

/// Represents the microstepping resolution for the DRV8825.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Microsteps {
    Full,
    Half,
    Quarter,
    Eighth,
    Sixteenth,
    ThirtySecond,
}

/// Represents the motor direction.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Forward,
    Backward,
}

/// A DRV8825 driver instance.
pub struct Drv8825<STEP, DIR, EN, M0, M1, M2> {
    step: STEP,
    dir: DIR,
    enable: EN,
    m0: M0,
    m1: M1,
    m2: M2,
}

impl<STEP, DIR, EN, M0, M1, M2, E> Drv8825<STEP, DIR, EN, M0, M1, M2>
where
    STEP: OutputPin<Error = E>,
    DIR: OutputPin<Error = E>,
    EN: OutputPin<Error = E>,
    M0: OutputPin<Error = E>,
    M1: OutputPin<Error = E>,
    M2: OutputPin<Error = E>,
{
    /// Creates a new DRV8825 driver from its control pins.
    pub fn new(step: STEP, dir: DIR, enable: EN, m0: M0, m1: M1, m2: M2) -> Self {
        Self { step, dir, enable, m0, m1, m2 }
    }

    /// Enables the driver (by pulling the nENABLE pin low).
    pub fn enable(&mut self) -> Result<(), E> {
        self.enable.set_low()
    }

    /// Disables the driver (by pulling the nENABLE pin high).
    pub fn disable(&mut self) -> Result<(), E> {
        self.enable.set_high()
    }

    /// Sets the motor direction.
    pub fn set_direction(&mut self, direction: Direction) -> Result<(), E> {
        let state = match direction {
            Direction::Forward => PinState::High, // Or Low, depending on wiring
            Direction::Backward => PinState::Low,
        };
        self.dir.set_state(state)
    }

    /// Performs one step by pulsing the STEP pin.
    /// Note: In a real application, you'll need a delay between high and low.
    /// This function is simplified for demonstration.
    pub fn step(&mut self) -> Result<(), E> {
        self.step.set_high()?;
        // A brief delay is required here in a real system.
        self.step.set_low()
    }

    /// Sets the microstepping resolution.
    pub fn set_microsteps(&mut self, microsteps: Microsteps) -> Result<(), E> {
        let (m0, m1, m2) = match microsteps {
            Microsteps::Full => (PinState::Low, PinState::Low, PinState::Low),
            Microsteps::Half => (PinState::High, PinState::Low, PinState::Low),
            Microsteps::Quarter => (PinState::Low, PinState::High, PinState::Low),
            Microsteps::Eighth => (PinState::High, PinState::High, PinState::Low),
            Microsteps::Sixteenth => (PinState::Low, PinState::Low, PinState::High),
            Microsteps::ThirtySecond => (PinState::High, PinState::Low, PinState::High),
        };
        self.m0.set_state(m0)?;
        self.m1.set_state(m1)?;
        self.m2.set_state(m2)
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::pin::{Mock as PinMock, State, Transaction};

    #[test]
    fn test_drv8825_enable_disable() {
        let step = PinMock::new(&[]);
        let dir = PinMock::new(&[]);
        let mut enable = PinMock::new(&[
            Transaction::set(State::Low),
            Transaction::set(State::High),
        ]);
        let m0 = PinMock::new(&[]);
        let m1 = PinMock::new(&[]);
        let m2 = PinMock::new(&[]);

        let mut driver = Drv8825::new(step, dir, enable, m0, m1, m2);
        driver.enable().unwrap();
        driver.disable().unwrap();

        driver.enable.done(); // Verify expectations
    }

    #[test]
    fn test_drv8825_set_microsteps() {
        let step = PinMock::new(&[]);
        let dir = PinMock::new(&[]);
        let enable = PinMock::new(&[]);
        let expectations = [
            // Set to 1/32
            (Transaction::set(State::High), Transaction::set(State::Low), Transaction::set(State::High)),
            // Set to Full
            (Transaction::set(State::Low), Transaction::set(State::Low), Transaction::set(State::Low)),
        ];

        let mut m0 = PinMock::new(&[expectations[0].0, expectations[1].0]);
        let mut m1 = PinMock::new(&[expectations[0].1, expectations[1].1]);
        let mut m2 = PinMock::new(&[expectations[0].2, expectations[1].2]);

        let mut driver = Drv8825::new(step, dir, enable, &mut m0, &mut m1, &mut m2);
        driver.set_microsteps(Microsteps::ThirtySecond).unwrap();
        driver.set_microsteps(Microsteps::Full).unwrap();

        m0.done();
        m1.done();
        m2.done();
    }
}

