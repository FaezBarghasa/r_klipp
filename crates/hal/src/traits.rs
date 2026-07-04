use embedded_hal::digital::ErrorType;
use embedded_hal::spi::Error as SpiError;
use embedded_hal::adc::Error as AdcError;

#[derive(Debug)]
pub enum CanError {
    // Define CAN error variants
}

#[derive(Debug)]
pub enum AdcChannel {
    // Define ADC channel variants
}

#[derive(Debug)]
pub struct CanFrame {
    // Define CAN frame structure
}

/// A timer for generating step pulses.
pub trait StepTimer {
    type Error: ErrorType;

    /// Sets the period of the timer in ticks.
    fn set_period(&mut self, ticks: u32) -> Result<(), Self::Error>;

    /// Enables DMA for this timer.
    fn enable_dma(&mut self) -> Result<(), Self::Error>;

    /// Gets the current counter value.
    fn get_counter(&self) -> Result<u32, Self::Error>;
}

/// A PWM output.
pub trait PwmOutput {
    type Error: ErrorType;

    /// Sets the duty cycle of the PWM output.
    fn set_duty_cycle(&mut self, duty: u16) -> Result<(), Self::Error>;

    /// Sets the frequency of the PWM output.
    fn set_frequency(&mut self, hz: u32) -> Result<(), Self::Error>;
}

/// A quadrature encoder interface.
pub trait QuadratureEncoder {
    type Error: ErrorType;

    /// Reads the current position of the encoder.
    fn read_position(&self) -> Result<i32, Self::Error>;

    /// Resets the encoder position to zero.
    fn reset(&mut self) -> Result<(), Self::Error>;

    /// Sets the hardware noise filter.
    fn set_filter(&mut self, samples: u8) -> Result<(), Self::Error>;
}

/// High-speed SPI for communication with stepper drivers.
pub trait HighSpeedSpi {
    /// Asynchronously transfers data over SPI.
    async fn transfer(&mut self, tx: &[u8], rx: &mut [u8]) -> Result<(), SpiError>;
}

/// ADC with DMA for continuous sampling.
pub trait AdcDma {
    /// Asynchronously reads ADC channels continuously into a buffer.
    async fn read_continuous(&mut self, channels: &[AdcChannel], buffer: &mut [u16]) -> Result<(), AdcError>;
}

/// A CAN bus interface.
pub trait CanBus {
    /// Asynchronously transmits a CAN frame.
    async fn transmit(&self, frame: &CanFrame) -> Result<(), CanError>;

    /// Asynchronously receives a CAN frame.
    async fn receive(&self) -> Result<CanFrame, CanError>;
}
