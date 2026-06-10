#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HostProcessor {
    QuadCoreCortexA53A7,
    QuadCoreCortexA72A76,
    QuadCoreCortexA53,
    DualCoreMipsXburst,
}

impl HostProcessor {
    pub fn description(&self) -> &'static str {
        match self {
            Self::QuadCoreCortexA53A7 => "Quad-Core Arm Cortex-A53/A7 (1.2 GHz - 1.5 GHz) - Good: A great starting point. Handles Klipper host duties for 1-2 basic printers easily.",
            Self::QuadCoreCortexA72A76 => "Quad-Core Arm Cortex-A72/A76 (1.5 GHz - 2.4 GHz) - Better: The sweet spot. Effortlessly manages Klipper, web interfaces, and a camera for 2-3 printers.",
            Self::QuadCoreCortexA53 => "Quad-Core Arm Cortex-A53 (1.4 GHz) - Integrated: An SBC is built right into the control board. This saves space and simplifies wiring.",
            Self::DualCoreMipsXburst => "Dual-Core MIPS XBurst (1.0 GHz) - Adequate: These are designed for \"turnkey\" setups. They run Klipper but have limited power for heavy multitasking.",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedHost {
    RaspberryPi3B,
    OrangePiZero2,
    LibreComputerBoardAmlS905xCc,
    RaspberryPi4,
    RaspberryPi5,
    RockPi4,
    MksSkipr,
    CrealityNebulaPad,
    BttPad7,
}

impl SupportedHost {
    pub fn processor(&self) -> HostProcessor {
        match self {
            Self::RaspberryPi3B
            | Self::OrangePiZero2
            | Self::LibreComputerBoardAmlS905xCc => HostProcessor::QuadCoreCortexA53A7,
            
            Self::RaspberryPi4 
            | Self::RaspberryPi5 
            | Self::RockPi4 => HostProcessor::QuadCoreCortexA72A76,
            
            Self::MksSkipr => HostProcessor::QuadCoreCortexA53,
            
            Self::CrealityNebulaPad 
            | Self::BttPad7 => HostProcessor::DualCoreMipsXburst,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::RaspberryPi3B => "Raspberry Pi 3B",
            Self::OrangePiZero2 => "Orange Pi Zero 2",
            Self::LibreComputerBoardAmlS905xCc => "Libre Computer Board AML-S905X-CC",
            Self::RaspberryPi4 => "Raspberry Pi 4 (BCM2711)",
            Self::RaspberryPi5 => "Raspberry Pi 5 (BCM2712)",
            Self::RockPi4 => "Rock Pi 4",
            Self::MksSkipr => "MKS SKIPR",
            Self::CrealityNebulaPad => "Creality Nebula Pad",
            Self::BttPad7 => "BTT Pad 7",
        }
    }
}