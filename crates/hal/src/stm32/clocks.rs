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
            config.rcc.sys = Sysclk::HSE;
        }
        ChipId::Stm32f407 => {
            config.rcc.hse = Some(Hse {
                freq: Hertz(8_000_000),
                mode: HseMode::Oscillator,
            });
            config.rcc.sys = Sysclk::HSE;
        }
        _ => {}
    }
    config
}

