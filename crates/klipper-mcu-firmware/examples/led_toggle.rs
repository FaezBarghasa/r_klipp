#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use defmt_rtt as _;
use embassy_executor::Spawner;
use embassy_stm32::{
    gpio::{Level, Output, Speed},
    Config,
};
use panic_probe as _;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let p = embassy_stm32::init(Config::default());
    defmt::info!("Ping example started! Toggling LED.");

    let mut led = Output::new(p.PB1, Level::High, Speed::Low); // Assuming LED is on PB1 for SKIPR

    // This is a simplified ping response.
    // A real implementation would listen on UART for a "ping" command
    // and then toggle the LED.
    loop {
        led.toggle();
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
    }
}
