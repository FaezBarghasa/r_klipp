use embedded_hal::digital::ErrorType;

/// Step timer trait for hardware timers generating stepper pulses
pub trait StepTimer {
    type Error;
    fn set_frequency(&mut self, freq_hz: u32) -> Result<(), Self::Error>;
    fn start(&mut self) -> Result<(), Self::Error>;
    fn stop(&mut self) -> Result<(), Self::Error>;
}

/// PWM output trait
pub trait PwmOutput {
    type Error;
    fn set_duty(&mut self, duty: u16) -> Result<(), Self::Error>;
    fn enable(&mut self) -> Result<(), Self::Error>;
    fn disable(&mut self) -> Result<(), Self::Error>;
}

/// Quadrature encoder interface
pub trait QuadratureEncoder {
    type Error;
    fn read_count(&self) -> i32;
    fn reset_count(&mut self);
}

/// ADC Channel trait
pub trait AdcChannel {
    type Error;
    fn read_sample(&mut self) -> Result<u16, Self::Error>;
}

/// ADC DMA Stream trait
pub trait AdcDma {
    type Error;
    fn start_streaming(&mut self, buffer: &mut [u16]) -> Result<(), Self::Error>;
}

/// High speed SPI bus for TMC drivers
pub trait HighSpeedSpi {
    type Error;
    fn transfer(&mut self, write: &[u8], read: &mut [u8]) -> Result<(), Self::Error>;
}

/// Abstracts CPU-level interrupt masking for creating critical sections.
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
    type Error;
    async fn read(&mut self) -> Result<WORD, Self::Error>;
}

/// An asynchronous SPI bus.
pub trait Spi {
    type Error;
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
    type Frame;
    type Error;
    async fn transmit(&mut self, frame: &Self::Frame) -> Result<(), Self::Error>;
    async fn receive(&mut self) -> Result<Self::Frame, Self::Error>;
}

