//! MCU Client
//!
//! Manages the serial connection to the microcontroller unit(s), handles
//! the Klipper binary protocol for sending commands and receiving responses,
//! and updates the shared printer state.

use crate::config::McuConfig;
use crate::gcode::McuCommand;
use crate::state::{PrinterState, PrinterStatus};
use anyhow::{Context, Result};
use parking_lot::Mutex;
use rand::Rng;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::Receiver;
use tokio::time::sleep;
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use tracing::{error, info, warn};

/// The main task for the real MCU client.
pub async fn run_mcu_client(
    config: McuConfig,
    mcu_rx: Receiver<McuCommand>,
    state: Arc<Mutex<PrinterState>>,
) {
    info!(
        "Attempting to connect to MCU at {} with baud rate {}",
        config.serial_port, config.baud_rate
    );

    loop {
        // Attempt to connect to the serial port.
        match tokio_serial::new(&config.serial_port, config.baud_rate).open_native_async() {
            Ok(port) => {
                info!("Successfully connected to MCU.");
                state.lock().status = PrinterStatus::Ready;
                state.lock().status_message = "Printer is ready".to_string();

                // If connection succeeds, run the communication loop.
                if let Err(e) = mcu_comm_loop(port, mcu_rx, state.clone()).await {
                    error!("MCU communication error: {}. Will attempt to reconnect.", e);
                }
            }
            Err(e) => {
                let error_msg = format!("Failed to connect to MCU: {}. Retrying in 5s.", e);
                warn!("{}", error_msg);
                let mut locked_state = state.lock();
                locked_state.status = PrinterStatus::Disconnected;
                locked_state.status_message = error_msg;
            }
        }

        // Wait before retrying connection.
        sleep(Duration::from_secs(5)).await;
    }
}

/// The main communication loop for reading from and writing to the MCU.
async fn mcu_comm_loop(
    _port: SerialStream,
    mut mcu_rx: Receiver<McuCommand>,
    _state: Arc<Mutex<PrinterState>>,
) -> Result<()> {
    // In a real implementation, you would have two tasks:
    // 1. A read task that uses a codec (like tokio_util::codec) to parse
    //    binary messages from the MCU and update the printer state.
    // 2. A write task that listens on `mcu_rx` and sends binary-encoded
    //    commands to the MCU.

    info!("MCU communication loop started.");
    loop {
        tokio::select! {
            Some(command) = mcu_rx.recv() => {
                info!("MCU Client received command: {:?}", command);
                // TODO: Encode the command into Klipper's binary protocol and
                // write it to the serial port.
                // For now, we just log it.
            }
            // A placeholder for the read side.
            // _ = port.read(&mut [0; 128]) => {
            //    // Decode message and update state
            // }
            else => {
                info!("MCU command channel closed. Exiting communication loop.");
                break;
            }
        }
    }
    Ok(())
}


/// The main task for the mock MCU client, used with `--mock-mcu`.
pub async fn run_mock_mcu(mut mcu_rx: Receiver<McuCommand>, state: Arc<Mutex<PrinterState>>) {
    info!("Mock MCU is running.");
    state.lock().status = PrinterStatus::Ready;
    state.lock().status_message = "Printer is ready (Mock MCU)".to_string();

    // Simulate periodic temperature updates.
    let temp_state = state.clone();
    tokio::spawn(async move {
        let mut rng = rand::thread_rng();
        loop {
            sleep(Duration::from_secs(2)).await;
            let mut locked_state = temp_state.lock();
            if let Some(extruder_temp) = locked_state.temperatures.get_mut("extruder") {
                extruder_temp.actual += rng.gen_range(-0.5..0.5);
            }
            if let Some(bed_temp) = locked_state.temperatures.get_mut("heater_bed") {
                bed_temp.actual += rng.gen_range(-0.5..0.5);
            }
        }
    });

    // Process incoming commands.
    while let Some(command) = mcu_rx.recv().await {
        info!("[Mock MCU] Received command: {:?}", command);
        sleep(Duration::from_millis(50)).await; // Simulate work
        info!("[Mock MCU] Command processed successfully.");
    }
}

