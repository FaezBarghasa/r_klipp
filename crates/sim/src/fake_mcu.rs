//! Simulated MCU Behavior
//!
//! Implements a minimal subset of MCU protocol behavior, including acknowledgments,
//! pings, step event acknowledgments, and simulated ADC temperature readings.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, info};

/// Represents a simulated Microcontroller Unit.
/// It listens on a Unix socket for commands from the host.
pub struct SimMcu {
    socket_path: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum McuCommand {
    Ping { sequence: u32 },
    GetConfig,
    GetStatus,
    AdcRead { pin: String },
    EmergencyStop,
    Move { steps: u32 },
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum McuResponse {
    Ack { sequence: u32 },
    Config { settings: String },
    Status { text: String },
    AdcValue { pin: String, value: f32 },
    StepEvent,
    Shutdown { reason: String },
}

impl SimMcu {
    /// Creates a new `SimMcu` that will listen on the given socket path.
    pub fn new(socket_path: &str) -> Self {
        SimMcu {
            socket_path: socket_path.to_string(),
        }
    }

    /// Runs the MCU simulation, listening for a single host connection.
    pub async fn run(&self) -> Result<()> {
        info!(path = %self.socket_path, "Starting simulated MCU...");
        let listener = tokio::net::UnixListener::bind(&self.socket_path)?;
        let (stream, _) = listener.accept().await?;
        info!("MCU simulator accepted host connection.");

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);
        let mut line = String::new();

        loop {
            tokio::select! {
                res = reader.read_line(&mut line) => {
                    if res? == 0 {
                        info!("Host disconnected.");
                        break;
                    }

                    let command: McuCommand = serde_json::from_str(line.trim())?;
                    debug!(?command, "MCU received command");

                    let response = self.handle_command(command).await;
                    let response_json = serde_json::to_string(&response)? + "\n";

                    writer.write_all(response_json.as_bytes()).await?;
                    debug!(?response, "MCU sent response");

                    line.clear();
                },
                _ = tokio::time::sleep(Duration::from_secs(5)) => {
                    // Simulate periodic events like temperature updates if needed
                }
            }
        }
        Ok(())
    }

    /// Handles a command from the host and generates a response.
    async fn handle_command(&self, command: McuCommand) -> McuResponse {
        match command {
            McuCommand::Ping { sequence } => McuResponse::Ack { sequence },
            McuCommand::GetConfig => McuResponse::Config {
                settings: "Simulated Config".to_string(),
            },
            McuCommand::GetStatus => McuResponse::Status {
                text: "Simulated Status: Ready".to_string(),
            },
            McuCommand::AdcRead { pin } if pin == "temp_sensor" => {
                // Simulate a temperature reading
                McuResponse::AdcValue {
                    pin,
                    value: 65.5,
                }
            }
            McuCommand::AdcRead { pin } => McuResponse::AdcValue { pin, value: 0.0 },
            McuCommand::EmergencyStop => McuResponse::Shutdown {
                reason: "Emergency Stop".to_string(),
            },
            McuCommand::Move { .. } => {
                // Acknowledge the move, then simulate a step event after a short delay
                tokio::spawn(async {
                    // This would typically be a more complex simulation
                });
                McuResponse::StepEvent
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::AsyncWriteExt;
    use tokio::net::UnixStream;

    #[tokio::test]
    async fn test_ping_ack() {
        let socket_path = "/tmp/sim_mcu_test_ping.sock";
        let _ = std::fs::remove_file(socket_path);
        let sim_mcu = SimMcu::new(socket_path);

        tokio::spawn(async move {
            sim_mcu.run().await.unwrap();
        });

        // Give the server a moment to start
        tokio::time::sleep(Duration::from_millis(50)).await;

        let mut stream = UnixStream::connect(socket_path).await.unwrap();
        let ping_cmd = McuCommand::Ping { sequence: 123 };
        let cmd_json = serde_json::to_string(&ping_cmd).unwrap() + "\n";
        stream.write_all(cmd_json.as_bytes()).await.unwrap();

        let mut reader = BufReader::new(stream);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await.unwrap();

        let response: McuResponse = serde_json::from_str(response_line.trim()).unwrap();
        match response {
            McuResponse::Ack { sequence } => assert_eq!(sequence, 123),
            _ => panic!("Unexpected response type"),
        }
        let _ = std::fs::remove_file(socket_path);
    }
}
