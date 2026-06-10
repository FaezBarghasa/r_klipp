use crate::firmware_support::McuArchitecture;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SupportedBoard {
    BigTreeTechOctopusV1_1,
    BigTreeTechOctopusPro,
    FysetcSpiderV2_2,
    FysetcSpiderV3_0,
    BigTreeTechSkr2,
    MksRobinNanoV3,
    BigTreeTechSkrV1_4Turbo,
    BigTreeTechSkrV1_3,
    BigTreeTechSkr3,
    BigTreeTechSkr3Ez,
    BigTreeTechOctopusMaxEz,
    BigTreeTechSkrPico,
    Duet3Mainboard6Hc,
    Duet3Mini5Plus,
}

impl SupportedBoard {
    pub fn architecture(&self) -> McuArchitecture {
        match self {
            Self::BigTreeTechOctopusV1_1 | Self::BigTreeTechOctopusPro => McuArchitecture::Stm32F4,
            Self::FysetcSpiderV2_2 | Self::FysetcSpiderV3_0 => McuArchitecture::Stm32F4,
            Self::BigTreeTechSkr2 | Self::MksRobinNanoV3 => McuArchitecture::Stm32F4,
            Self::BigTreeTechSkrV1_4Turbo | Self::BigTreeTechSkrV1_3 => McuArchitecture::Lpc176x,
            Self::BigTreeTechSkr3 | Self::BigTreeTechSkr3Ez | Self::BigTreeTechOctopusMaxEz => McuArchitecture::Stm32H7,
            Self::BigTreeTechSkrPico => McuArchitecture::Rp2040,
            Self::Duet3Mainboard6Hc | Self::Duet3Mini5Plus => McuArchitecture::AtSam,
        }
    }

    pub fn name(&self) -> &'static str {
        match self {
            Self::BigTreeTechOctopusV1_1 => "BigTreeTech Octopus V1.1",
            Self::BigTreeTechOctopusPro => "BigTreeTech Octopus Pro",
            Self::FysetcSpiderV2_2 => "Fysetc Spider V2.2",
            Self::FysetcSpiderV3_0 => "Fysetc Spider V3.0",
            Self::BigTreeTechSkr2 => "BigTreeTech SKR 2",
            Self::MksRobinNanoV3 => "MKS Robin Nano V3",
            Self::BigTreeTechSkrV1_4Turbo => "BigTreeTech SKR V1.4 Turbo",
            Self::BigTreeTechSkrV1_3 => "BigTreeTech SKR V1.3/V1.4",
            Self::BigTreeTechSkr3 => "BigTreeTech SKR 3",
            Self::BigTreeTechSkr3Ez => "BigTreeTech SKR 3 EZ",
            Self::BigTreeTechOctopusMaxEz => "BigTreeTech Octopus Max EZ",
            Self::BigTreeTechSkrPico => "BigTreeTech SKR Pico",
            Self::Duet3Mainboard6Hc => "Duet 3 Mainboard 6HC",
            Self::Duet3Mini5Plus => "Duet 3 Mini 5+",
        }
    }
}