//! # Embassy Main
//!
//! This is the main entry point for the firmware when using the Embassy executor.
//! It initializes the hardware, spawns all the concurrent tasks, and then lets the
//! executor take over.

use crate::{adc, heater, proto_bridge, stepper};
use boards::stm32f407::pins::BoardPins;
use embassy_executor::Spawner;
use embassy_stm32::Config;

/// The main asynchronous function that sets up and runs the firmware.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::info!("Initializing Klipper MCU Firmware...");

    // Board-specific configuration and initialization.
    let mut config = Config::default();
    // Configure clocks here if needed, e.g., for high-speed stepping.
    // config.rcc.hse = Some(embassy_stm32::rcc::Hse { ... });
    let p = embassy_stm32::init(config);
    let board_pins = BoardPins::new(p);

    // Spawn all the concurrent tasks.
    // The spawner is responsible for running these tasks in the background.
    spawner.spawn(proto_bridge::proto_task(p.USART1, board_pins.uart_rx, board_pins.uart_tx)).unwrap();
    spawner.spawn(stepper::stepper_task()).unwrap();
    spawner.spawn(adc::adc_task(p.ADC1, board_pins.temp_extruder)).unwrap();
    spawner.spawn(heater::heater_task()).unwrap();
    spawner.spawn(led_task(board_pins.led.into())).unwrap();

    defmt::info!("Initialization complete. All tasks are running.");
}

/// A simple task to blink the LED, indicating that the firmware is running.
#[embassy_executor::task]
async fn led_task(led_pin: embassy_stm32::gpio::AnyPin) {
    use embassy_stm32::gpio::{Level, Output, Speed};
    use embassy_time::{Duration, Timer};
    let mut led = Output::new(led_pin, Level::High, Speed::Low);

    loop {
        led.toggle();
        Timer::after(Duration::from_millis(500)).await;
    }
}
