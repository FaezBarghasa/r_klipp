use embassy_stm32::rcc::*;
use embassy_stm32::time::Hertz;
use embassy_stm32::Config;

pub enum ChipId {
    Stm32f103,
    Stm32f407,
    Stm32g031,
    Stm32c031,
    Stm32l476,
    Stm32wb55,
}

pub fn configure_clocks(chip: ChipId) -> Config {
    let mut config = Config::default();
    match chip {
        ChipId::Stm32f103 => {
            config.rcc.hse = Some(Hse {
                freq: Hertz(8_000_000),
                mode: HseMode::Oscillator,
            });
            config.rcc.pll = Some(Pll {
                source: PllSource::HSE,
                prediv: PllPreDiv::DIV1,
                mul: PllMul::MUL9,
            });
            config.rcc.sys = Sysclk::PLL;
            config.rcc.ahb_pre = AHBPrescaler::DIV1;
            config.rcc.apb1_pre = APBPrescaler::DIV2;
            config.rcc.apb2_pre = APBPrescaler::DIV1;
            config.rcc.usb_pre = UsbPrescaler::DIV1_5;
        }
        ChipId::Stm32f407 => {
            config.rcc.hse = Some(Hse {
                freq: Hertz(8_000_000),
                mode: HseMode::Oscillator,
            });
            config.rcc.pll = Some(Pll {
                source: PllSource::HSE,
                prediv: PllPreDiv::DIV4,
                mul: PllMul::MUL168,
                divp: Some(PllPDiv::DIV2),
                divq: Some(PllQDiv::DIV7),
                divr: None,
            });
            config.rcc.sys = Sysclk::PLL;
            config.rcc.ahb_pre = AHBPrescaler::DIV1;
            config.rcc.apb1_pre = APBPrescaler::DIV4;
            config.rcc.apb2_pre = APBPrescaler::DIV2;
        }
        ChipId::Stm32g031 | ChipId::Stm32c031 => {
            config.rcc.hsi = true;
            config.rcc.pll = Some(Pll {
                source: PllSource::HSI,
                prediv: PllPreDiv::DIV1,
                mul: PllMul::MUL8,
                divp: None,
                divq: None,
                divr: Some(PllRDiv::DIV2),
            });
            config.rcc.sys = Sysclk::PLL;
            config.rcc.ahb_pre = AHBPrescaler::DIV1;
            config.rcc.apb1_pre = APBPrescaler::DIV1;
            config.rcc.apb2_pre = APBPrescaler::DIV1;
        }
        _ => {
            // Default configuration
        }
    }
    config
}
