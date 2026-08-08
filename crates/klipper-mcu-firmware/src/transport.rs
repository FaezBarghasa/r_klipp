#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransportError {
    BufferFull,
    BufferEmpty,
    FramingError,
    OverrunError,
    HardwareError,
}

/// A generic interface for MCU communication protocols.
pub trait Transport {
    fn write(&mut self, data: &[u8]) -> Result<usize, TransportError>;
    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, TransportError>;
    fn flush(&mut self) -> Result<(), TransportError>;
}

#[derive(Debug, Clone, Copy)]
pub struct SerialConfig {
    pub baud_rate: u32,
}

/// Represents a Universal Synchronous/Asynchronous Receiver-Transmitter (USART/UART) connection.
pub struct SerialConnection<P> {
    pub peripheral: P,
    pub config: SerialConfig,
}

impl<P> SerialConnection<P> {
    pub const fn new(peripheral: P, config: SerialConfig) -> Self {
        Self { peripheral, config }
    }
}

impl<P: Transport> Transport for SerialConnection<P> {
    fn write(&mut self, data: &[u8]) -> Result<usize, TransportError> {
        self.peripheral.write(data)
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, TransportError> {
        self.peripheral.read(buffer)
    }

    fn flush(&mut self) -> Result<(), TransportError> {
        self.peripheral.flush()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum McuError {
    ClockSpeedTooLow,
}

/// Represents a supported microcontroller unit (MCU) and its common architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SupportedMcu {
    Stm32F103,
    Stm32G0B1,
    Stm32G071,
    Stm32F401,
    Stm32F407,
    Stm32F446,
    Stm32H723,
    Lpc1768,
    Lpc1769,
    Rp2040,
    Sam3X8E,
    Sam4E8E,
    Sam4S,
    SamE54,
    Esp32,
}

impl SupportedMcu {
    /// Returns the typical clock speed in Hz for the specified MCU.
    pub const fn default_clock_hz(&self) -> u32 {
        match self {
            Self::Stm32F103 => 72_000_000,
            Self::Stm32G0B1 | Self::Stm32G071 => 64_000_000,
            Self::Stm32F401 => 84_000_000,
            Self::Stm32F407 => 168_000_000,
            Self::Stm32F446 => 180_000_000,
            Self::Stm32H723 => 550_000_000,
            Self::Lpc1768 => 100_000_000,
            Self::Lpc1769 => 120_000_000,
            Self::Rp2040 => 133_000_000,
            Self::Sam3X8E => 84_000_000,
            Self::Sam4E8E | Self::Sam4S | Self::SamE54 => 120_000_000,
            Self::Esp32 => 240_000_000,
        }
    }
}

/// Represents the MCU firmware configuration and transport layer.
pub struct McuFirmware<T> {
    pub transport: T,
    pub mcu: Option<SupportedMcu>,
    pub clock_hz: u32,
}

impl<T: Transport> McuFirmware<T> {
    /// Creates a new `McuFirmware` instance.
    ///
    /// Only supports microcontrollers with a clock speed of 1 MHz (1,000,000 Hz) or more.
    pub fn new(transport: T, clock_hz: u32) -> Result<Self, McuError> {
        if clock_hz < 1_000_000 {
            return Err(McuError::ClockSpeedTooLow);
        }
        Ok(Self { transport, mcu: None, clock_hz })
    }

    /// Creates a new `McuFirmware` instance based on a supported MCU's default configuration.
    pub fn from_mcu(transport: T, mcu: SupportedMcu) -> Result<Self, McuError> {
        let clock_hz = mcu.default_clock_hz();
        if clock_hz < 1_000_000 {
            return Err(McuError::ClockSpeedTooLow);
        }
        Ok(Self { transport, mcu: Some(mcu), clock_hz })
    }

    pub fn write(&mut self, data: &[u8]) -> Result<usize, TransportError> {
        self.transport.write(data)
    }

    pub fn read(&mut self, buffer: &mut [u8]) -> Result<usize, TransportError> {
        self.transport.read(buffer)
    }

    pub fn flush(&mut self) -> Result<(), TransportError> {
        self.transport.flush()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct CanConfig {
    pub bit_rate: u32,
    pub node_id: u32,
}

/// Represents a Controller Area Network (CAN) connection.
pub struct CanConnection<P> {
    pub peripheral: P,
    pub config: CanConfig,
}

impl<P> CanConnection<P> {
    pub const fn new(peripheral: P, config: CanConfig) -> Self {
        Self { peripheral, config }
    }
}

impl<P: Transport> Transport for CanConnection<P> {
    fn write(&mut self, data: &[u8]) -> Result<usize, TransportError> {
        self.peripheral.write(data)
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, TransportError> {
        self.peripheral.read(buffer)
    }

    fn flush(&mut self) -> Result<(), TransportError> {
        self.peripheral.flush()
    }
}

#[derive(Debug, Clone, Copy)]
pub struct UsbConfig {
    pub vendor_id: u16,
    pub product_id: u16,
}

/// Represents a Universal Serial Bus (USB) connection.
pub struct UsbConnection<P> {
    pub peripheral: P,
    pub config: UsbConfig,
}

impl<P> UsbConnection<P> {
    pub const fn new(peripheral: P, config: UsbConfig) -> Self {
        Self { peripheral, config }
    }
}

impl<P: Transport> Transport for UsbConnection<P> {
    fn write(&mut self, data: &[u8]) -> Result<usize, TransportError> {
        self.peripheral.write(data)
    }

    fn read(&mut self, buffer: &mut [u8]) -> Result<usize, TransportError> {
        self.peripheral.read(buffer)
    }

    fn flush(&mut self) -> Result<(), TransportError> {
        self.peripheral.flush()
    }
}