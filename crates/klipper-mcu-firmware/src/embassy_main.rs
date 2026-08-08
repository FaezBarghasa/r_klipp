//! # Embassy Main
//!
//! This is the main entry point for the firmware when using the Embassy executor.
//! It initializes the hardware, spawns all the concurrent tasks, and then lets the
//! executor take over.

use crate::task_spawner::TaskSpawner;
use embassy_executor::Spawner;
use embassy_stm32::Config;
use embassy_time::{Duration, Timer};

/// The main asynchronous function that sets up and runs the firmware.
#[embassy_executor::main]
async fn main(spawner: Spawner) {
    defmt::info!("Initializing Klipper MCU Firmware...");

    // Board-specific configuration and initialization.
    let config = Config::default();
    let _p = embassy_stm32::init(config);
    
    let _task_spawner = TaskSpawner::new(spawner);

    // TODO: Initialize and spawn actors here.

    defmt::info!("Initialization complete. All actors are running.");

    // The executor will now run the spawned tasks forever.
    // We can add a heartbeat or monitoring loop here if needed.
    loop {
        // For now, just yield to the executor.
        Timer::after(Duration::from_secs(1)).await;
    }
}
