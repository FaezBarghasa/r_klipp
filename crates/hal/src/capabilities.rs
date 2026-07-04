/// Enum for the core type.
#[derive(Debug, PartialEq, Eq)]
pub enum CoreType {
    M0,
    M0Plus,
    M3,
    M4F,
    M7F,
}

/// A trait to discover chip capabilities at compile time.
pub trait ChipCapabilities {
    /// Whether the chip has a floating-point unit.
    const HAS_FPU: bool;
    /// Whether the chip has a CORDIC coprocessor.
    const HAS_CORDIC: bool;
    /// Whether the chip has an FMAC (filter mathematical accelerator) unit.
    const HAS_FMAC: bool;
    /// Whether the chip has FDCAN (CAN with flexible data-rate).
    const HAS_FDCAN: bool;
    /// The number of hardware timers.
    const NUM_TIMERS: usize;
    /// The number of DMA channels.
    const NUM_DMA_CHANNELS: usize;
    /// The size of RAM in bytes.
    const RAM_SIZE: usize;
    /// The maximum clock frequency in Hz.
    const MAX_CLOCK_HZ: u32;
    /// The core type.
    const CORE_TYPE: CoreType;
}
