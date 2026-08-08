#![cfg_attr(feature = "async", feature(async_fn_in_trait))]

use embedded_hal::digital::ErrorType;
use embedded_hal::spi::Error as SpiError;
use embedded_hal::adc::Error as AdcError;
use embedded_hal::can::Frame as CanFrame;
use embedded_hal::can::Error as CanError;

/// Abstracts CPU-level interrupt masking for creating critical sections.
/// This allows for compatibility with both bare-metal (cortex_m::interrupt::free)
/// and simulated environments (std::sync::Mutex).
pub trait InterruptMasker {
    /// Disables all interrupts and returns a token that will re-enable them
    /// when it goes out of scope.
    fn disable_interrupts(&self) -> impl Drop;

    /// Enables all interrupts.
    fn enable_interrupts(&self);
}

/// A generic asynchronous timer.
pub trait Timer {
    async fn delay_us(&mut self, us: u32);
    async fn delay_ms(&mut self, ms: u32);
}

/// An asynchronous PWM output.
pub trait Pwm {
    type Error: ErrorType;
    async fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error>;
    async fn set_frequency(&mut self, hz: u32) -> Result<(), Self::Error>;
    async fn enable(&mut self) -> Result<(), Self::Error>;
    async fn disable(&mut self) -> Result<(), Self::Error>;
}

/// An asynchronous ADC channel.
pub trait Adc<WORD> {
    type Error: AdcError;
    async fn read(&mut self) -> Result<WORD, Self::Error>;
}

/// An asynchronous SPI bus.
pub trait Spi {
    type Error: SpiError;
    async fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<(), Self::Error>;
    async fn write(&mut self, tx: &[u8]) -> Result<(), Self::Error>;
}

/// An asynchronous UART/serial port.
pub trait Uart {
    type Error;
    async fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;
    async fn flush(&mut self) -> Result<(), Self::Error>;
}

/// An asynchronous GPIO pin.
pub trait Gpio {
    type Error: ErrorType;
    async fn set_high(&mut self) -> Result<(), Self::Error>;
    async fn set_low(&mut self) -> Result<(), Self::Error>;
    async fn is_high(&self) -> Result<bool, Self::Error>;
    async fn is_low(&self) -> Result<bool, Self::Error>;
}

/// An asynchronous DMA transfer.
pub trait Dma {
    type Error;
    async fn transfer(&mut self, src: &[u8], dst: &mut [u8], len: usize) -> Result<(), Self::Error>;
}

/// An asynchronous CAN bus interface.
pub trait Can {
    type Frame: CanFrame;
    type Error: CanError;
    async fn transmit(&mut self, frame: &Self::Frame) -> Result<(), Self::Error>;
    async fn receive(&mut self) -> Result<Self::Frame, Self::Error>;
}
