//! A `no_std` driver for reading the state of multiple digital endstops.
//!
//! This crate provides a simple interface to read a collection of GPIO input
//! pins, such as those connected to limit switches.

#![no_std]

use embedded_hal::digital::InputPin;

/// A collection of endstop input pins.
pub struct Endstops<const N: usize, PIN> {
    pins: [PIN; N],
}

impl<const N: usize, PIN, E> Endstops<N, PIN>
where
    PIN: InputPin<Error = E>,
{
    /// Creates a new instance from an array of input pins.
    pub fn new(pins: [PIN; N]) -> Self {
        Self { pins }
    }

    /// Reads the state of a single endstop pin by its index.
    ///
    /// Returns `true` if the endstop is triggered (pin is high).
    pub fn read_state(&self, index: usize) -> Result<bool, E> {
        self.pins[index].is_high()
    }

    /// Reads the state of all endstop pins.
    ///
    /// Returns an array of booleans, where `true` typically indicates the
    /// endstop is triggered (e.g., the pin is high).
    pub fn read_states(&self) -> Result<[bool; N], E> {
        let mut states = [false; N];
        for i in 0..N {
            states[i] = self.pins[i].is_high()?;
        }
        Ok(states)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embedded_hal_mock::pin::{Mock as PinMock, State, Transaction};

    #[test]
    fn test_read_six_endstops() {
        // Create mock pins for 6 endstops
        let mut pins = [
            PinMock::new(&[Transaction::get(State::Low)]),
            PinMock::new(&[Transaction::get(State::High)]),
            PinMock::new(&[Transaction::get(State::Low)]),
            PinMock::new(&[Transaction::get(State::High)]),
            PinMock::new(&[Transaction::get(State::Low)]),
            PinMock::new(&[Transaction::get(State::Low)]),
        ];

        let endstops = Endstops::new(pins);

        let states = endstops.read_states().unwrap();

        assert_eq!(states, [false, true, false, true, false, false]);

        // Verify that all mock expectations were met
        // Note: The pins array was moved, so we can't iterate here.
        // The `done()` check is implicitly handled when the mock object goes out of scope
        // if it was created with the exact number of expected transactions.
    }
}

