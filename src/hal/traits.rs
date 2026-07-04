//! Hardware Abstraction Layer (HAL) traits for r_klipp.
//! These traits define the interface for interacting with hardware peripherals.
//! They are designed to be implementable for both real hardware and simulators.
//! This file corresponds to Task 1.1 of the development plan.

#![no_std]

// Note: This code requires the `async-trait` crate.
// Add `async-trait = "0.1"` to your Cargo.toml.
use async_trait::async_trait;

/// A trait for timers that can provide delays.
#[async_trait]
pub trait Timer {
    /// Asynchronously waits for a specified number of milliseconds.
    async fn delay_ms(&mut self, ms: u32);
}

/// A trait for Pulse-Width Modulation (PWM) peripherals.
#[async_trait]
pub trait Pwm {
    /// Sets the duty cycle of the PWM output.
    fn set_duty_cycle(&mut self, duty: u16);
    /// Returns the maximum possible duty cycle value.
    fn get_max_duty(&self) -> u16;
}

/// A trait for Analog-to-Digital Converters (ADCs).
#[async_trait]
pub trait Adc<WORD> {
    /// The type of error that can occur during a read.
    type Error;
    /// Reads a single sample from the ADC.
    async fn read(&mut self) -> Result<WORD, Self::Error>;
}

/// A trait for Serial Peripheral Interface (SPI) peripherals.
#[async_trait]
pub trait Spi {
    /// The type of error that can occur during a transfer.
    type Error;
    /// Writes a slice of words to the SPI bus.
    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error>;
    /// Writes and reads words to/from the SPI bus simultaneously.
    async fn transfer<'w>(&mut self, read: &'w mut [u8], write: &[u8]) -> Result<(), Self::Error>;
}

/// A trait for Universal Asynchronous Receiver-Transmitter (UART) peripherals.
#[async_trait]
pub trait Uart {
    /// The type of error that can occur during a transfer.
    type Error;
    /// Writes a slice of words to the UART.
    async fn write(&mut self, words: &[u8]) -> Result<(), Self::Error>;
    /// Reads words from the UART into a buffer.
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;
}

/// A trait for General-Purpose Input/Output (GPIO) pins.
pub trait Gpio {
    /// The type of error that can occur.
    type Error;
    /// Sets the pin to a high logic level.
    fn set_high(&mut self) -> Result<(), Self::Error>;
    /// Sets the pin to a low logic level.
    fn set_low(&mut self) -> Result<(), Self::Error>;
    /// Returns `true` if the pin is at a high logic level.
    fn is_high(&self) -> Result<bool, Self::Error>;
    /// Returns `true` if the pin is at a low logic level.
    fn is_low(&self) -> Result<bool, Self::Error>;
}

/// A trait for Direct Memory Access (DMA) peripherals.
#[async_trait]
pub trait Dma {
    // This is a simplified DMA trait. A real-world implementation would be
    // more complex, likely involving channel/stream configuration, different
    // transfer modes (e.g., circular), and more granular control.
    /// The type of error that can occur during a transfer.
    type Error;
    /// Performs a memory-to-memory transfer.
    async fn transfer(&mut self, from: &[u8], to: &mut [u8]) -> Result<(), Self::Error>;
}

/// A trait for masking and unmasking interrupts.
/// This provides an abstraction over `cortex_m::interrupt::free` for simulator compatibility.
pub trait InterruptMasker {
    /// Disables interrupts globally.
    fn disable_interrupts(&self);
    /// Enables interrupts globally.
    fn enable_interrupts(&self);
}
