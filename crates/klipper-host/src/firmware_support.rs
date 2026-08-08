#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum McuArchitecture {
    Stm32F4,
    Lpc176x,
    Stm32H7,
    Rp2040,
    AtSam,
}

impl McuArchitecture {
    pub fn max_clock_speed_mhz(&self) -> u32 {
        match self {
            Self::Stm32F4 => 180,
            Self::Lpc176x => 120,
            Self::Stm32H7 => 550,
            Self::Rp2040 => 133,
            Self::AtSam => 300,
        }
    }

    pub fn description(&self) -> &'static str {
        match self {
            Self::Stm32F4 => "STM32F4 Series (168 MHz - 180 MHz) - Highly reliable, standard for Klipper.",
            Self::Lpc176x => "NXP LPC176x Series (100 MHz - 120 MHz) - Classic 32-bit architecture, rock solid legacy support.",
            Self::Stm32H7 => "STM32H7 Series (480 MHz - 550 MHz) - Extreme performance, requires newer Klipper versions.",
            Self::Rp2040 => "RP2040 Series (133 MHz Dual-Core) - Raspberry Pi silicon, highly compact.",
            Self::AtSam => "ATSAM Series (120 MHz - 300 MHz) - Premium tier, robust hardware support.",
        }
    }
}