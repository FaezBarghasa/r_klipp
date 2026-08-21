//! A `no_std` driver for reading and handling hardware interrupt endstops.
//!
//! Provides both polled and asynchronous/interrupt-driven EXTI limit switch monitoring
//! with atomic trigger recording and emergency motor halt triggers.

#![no_std]

use embedded_hal::digital::InputPin;
use core::sync::atomic::{AtomicBool, AtomicU8, Ordering};

/// Interrupt-safe endstop trigger registry (e.g. for EXTI line interrupts)
pub struct ExtiEndstopManager {
    triggered_mask: AtomicU8,
    active_high: bool,
    halt_steppers_on_trigger: AtomicBool,
}

impl ExtiEndstopManager {
    pub const fn new(active_high: bool) -> Self {
        Self {
            triggered_mask: AtomicU8::new(0),
            active_high,
            halt_steppers_on_trigger: AtomicBool::new(true),
        }
    }

    /// Called directly inside EXTI ISR. Executes in <100ns to flag limit hit.
    #[inline(always)]
    pub fn handle_exti_interrupt(&self, pin_index: u8, pin_is_high: bool) {
        let is_triggered = if self.active_high { pin_is_high } else { !pin_is_high };
        if is_triggered && pin_index < 8 {
            self.triggered_mask.fetch_or(1 << pin_index, Ordering::SeqCst);
        }
    }

    #[inline(always)]
    pub fn is_triggered(&self, pin_index: u8) -> bool {
        if pin_index < 8 {
            (self.triggered_mask.load(Ordering::Acquire) & (1 << pin_index)) != 0
        } else {
            false
        }
    }

    #[inline(always)]
    pub fn clear(&self) {
        self.triggered_mask.store(0, Ordering::Release);
    }

    #[inline(always)]
    pub fn should_halt(&self) -> bool {
        self.halt_steppers_on_trigger.load(Ordering::Relaxed) && self.triggered_mask.load(Ordering::Relaxed) != 0
    }
}

/// A collection of endstop input pins for polling.
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
    pub fn read_state(&mut self, index: usize) -> Result<bool, E> {
        self.pins[index].is_high()
    }

    /// Reads the state of all endstop pins.
    #[allow(clippy::needless_range_loop)]
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

    #[test]
    fn test_exti_endstop_interrupt_handling() {
        let manager = ExtiEndstopManager::new(true);
        assert!(!manager.is_triggered(0));
        assert!(!manager.should_halt());

        // EXTI ISR fires on Pin 0
        manager.handle_exti_interrupt(0, true);
        assert!(manager.is_triggered(0));
        assert!(manager.should_halt());
        assert!(!manager.is_triggered(1));

        manager.clear();
        assert!(!manager.is_triggered(0));
        assert!(!manager.should_halt());
    }
}
