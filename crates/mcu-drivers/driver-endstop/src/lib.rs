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
    pub fn read_state(&mut self, index: usize) -> Result<bool, E> {
        self.pins[index].is_high()
    }

    /// Reads the state of all endstop pins.
    ///
    /// Returns an array of booleans, where `true` typically indicates the
    /// endstop is triggered (e.g., the pin is high).
    pub fn read_states(&mut self) -> Result<[bool; N], E> {
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
    use embedded_hal_mock::eh1::digital::{Mock as PinMock, State, Transaction};

    #[test]
    fn test_read_six_endstops() {
        // Create mock pins for 6 endstops
        let mut p0 = PinMock::new(&[Transaction::get(State::Low)]);
        let mut p1 = PinMock::new(&[Transaction::get(State::High)]);
        let mut p2 = PinMock::new(&[Transaction::get(State::Low)]);
        let mut p3 = PinMock::new(&[Transaction::get(State::High)]);
        let mut p4 = PinMock::new(&[Transaction::get(State::Low)]);
        let mut p5 = PinMock::new(&[Transaction::get(State::Low)]);

        let mut endstops = Endstops::new([
            &mut p0,
            &mut p1,
            &mut p2,
            &mut p3,
            &mut p4,
            &mut p5,
        ]);

        let states = endstops.read_states().unwrap();

        assert_eq!(states, [false, true, false, true, false, false]);

        p0.done();
        p1.done();
        p2.done();
        p3.done();
        p4.done();
        p5.done();
    }
}

