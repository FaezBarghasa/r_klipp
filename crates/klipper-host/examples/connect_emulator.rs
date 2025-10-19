//! In-Process Host and MCU Emulator
//!
//! This example runs a simplified Klipper host and a simulated MCU in the same
//! process, communicating over an in-memory channel that emulates a serial port.
//! This is useful for testing the communication protocol without needing hardware.

use anyhow::Result;
use std::time::Duration;
use tokio::io::{self, AsyncReadExt, AsyncWriteExt};
use tokio::sync::mpsc;
use tokio::time::sleep;
use tracing::info;

/// A mock MCU that reads commands from a channel and writes back responses.
async fn run_mock_mcu(mut host_to_mcu_rx: impl AsyncReadExt + Unpin, mut mcu_to_host_tx: impl AsyncWriteExt + Unpin) {
    info!("[MCU Emulator] Started.");
    let mut buf = [0; 128];
    loop {
        match host_to_mcu_rx.read(&mut buf).await {
            Ok(0) => {
                info!("[MCU Emulator] Host disconnected.");
                break;
            }
            Ok(n) => {
                let received = &buf[..n];
                info!("[MCU Emulator] Received bytes: {:?}", received);
                // Echo back a simple "ok" response.
                if mcu_to_host_tx.write_all(b"ok\n").await.is_err() {
                    info!("[MCU Emulator] Host disconnected.");
                    break;
                }
            }
            Err(e) => {
                info!("[MCU Emulator] Error reading from host: {}", e);
                break;
            }
        }
    }
}

/// A simplified host application that sends commands.
async fn run_host(mut host_to_mcu_tx: impl AsyncWriteExt + Unpin, mut mcu_to_host_rx: impl AsyncReadExt + Unpin) {
    info!("[Host] Started.");
    let mut response_buf = [0; 128];
    for i in 0..5 {
        let command = format!("MOVE {}", i);
        info!("[Host] Sending command: {}", command);
        if host_to_mcu_tx.write_all(command.as_bytes()).await.is_err() {
            info!("[Host] MCU disconnected.");
            break;
        }

        // Wait for a response from the MCU.
        match mcu_to_host_rx.read(&mut response_buf).await {
            Ok(n) => info!("[Host] Received response: {}", String::from_utf8_lossy(&response_buf[..n]).trim()),
            Err(_) => {
                info!("[Host] MCU disconnected.");
                break;
            }
        }
        sleep(Duration::from_secs(1)).await;
    }
    info!("[Host] Finished.");
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();
    info!("Starting host and MCU emulator example...");

    // Create a duplex stream to act as an in-memory serial port.
    let (host_stream, mcu_stream) = io::duplex(1024);
    let (mcu_reader, mcu_writer) = io::split(mcu_stream);
    let (host_reader, host_writer) = io::split(host_stream);

    // Spawn the MCU and host tasks.
    let mcu_handle = tokio::spawn(run_mock_mcu(mcu_reader, mcu_writer));
    let host_handle = tokio::spawn(run_host(host_writer, host_reader));

    // Wait for both tasks to complete.
    let _ = tokio::try_join!(mcu_handle, host_handle)?;
    Ok(())
}

