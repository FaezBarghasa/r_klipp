use anyhow::{anyhow, Result};
use async_trait::async_trait;
use cobs::{decode_in_place, encode_in_place};
use log::{error, info, warn};
use postcard::{from_bytes, to_vec_cobs};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::sleep;
use tokio_serial::{ClearBuffer, SerialPortBuilderExt, SerialStream};

use crate::api::MachineState; // Assuming MachineState is defined in api module

// --- Postcard Message Definitions ---
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum McuToHost {
    Telemetry(Telemetry),
    Response(Response),
    Error(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum HostToMcu {
    GCode(String),
    Command(Command),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Telemetry {
    pub nozzle_temp: f32,
    pub bed_temp: f32,
    pub x_pos: f32,
    pub y_pos: f32,
    pub z_pos: f32,
    // Add more telemetry fields as needed
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Response {
    Ok,
    Value(String),
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Command {
    SetToolTemp(f32),
    SetBedTemp(f32),
    Home(Axis),
    Move { x: Option<f32>, y: Option<f32>, z: Option<f32> },
    // Add more commands
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
    All,
}

// --- Serial Bridge Implementation ---
pub struct SerialBridge {
    port_path: String,
    baud_rate: u32,
    telemetry_broadcaster: broadcast::Sender<serde_json::Value>,
    mcu_cmd_receiver: mpsc::Receiver<HostToMcu>,
    machine_state: Arc<RwLock<MachineState>>,
}

impl SerialBridge {
    pub fn new(
        port_path: String,
        baud_rate: u32,
        telemetry_broadcaster: broadcast::Sender<serde_json::Value>,
        mcu_cmd_receiver: mpsc::Receiver<HostToMcu>,
        machine_state: Arc<RwLock<MachineState>>,
    ) -> Self {
        Self {
            port_path,
            baud_rate,
            telemetry_broadcaster,
            mcu_cmd_receiver,
            machine_state,
        }
    }

    pub async fn run(mut self) -> Result<()> {
        info!("Starting SerialBridge task.");
        loop {
            match self.connect().await {
                Ok(mut port) => {
                    info!("Connected to serial port: {}", self.port_path);
                    let (mut reader, mut writer) = tokio::io::split(port);

                    let read_loop = self.read_loop(&mut reader);
                    let write_loop = self.write_loop(&mut writer);

                    tokio::select! {
                        _ = read_loop => {
                            warn!("Serial read loop ended. Attempting reconnect...");
                        }
                        _ = write_loop => {
                            warn!("Serial write loop ended. Attempting reconnect...");
                        }
                    }
                }
                Err(e) => {
                    error!("Failed to connect to serial port: {}. Retrying in 5 seconds...", e);
                    sleep(Duration::from_secs(5)).await;
                }
            }
        }
    }

    async fn connect(&self) -> Result<SerialStream> {
        tokio_serial::new(&self.port_path, self.baud_rate)
            .open_native_async()
            .map_err(|e| anyhow!("Failed to open serial port: {}", e))
    }

    async fn read_loop(&self, reader: &mut (impl AsyncReadExt + Unpin)) -> Result<()> {
        let mut buf = vec![0u8; 256]; // Max postcard message size
        let mut cobs_buf = vec![0u8; 256]; // Buffer for COBS decoding
        let mut read_pos = 0;

        loop {
            let bytes_read = match reader.read(&mut buf[read_pos..]).await {
                Ok(0) => {
                    warn!("Serial port closed unexpectedly (read).");
                    return Err(anyhow!("Serial port closed"));
                }
                Ok(n) => n,
                Err(ref e) if e.kind() == ErrorKind::Interrupted => continue,
                Err(e) => {
                    error!("Serial read error: {}", e);
                    return Err(anyhow!("Serial read error: {}", e));
                }
            };

            read_pos += bytes_read;

            // Process received bytes for COBS frames
            while let Some(frame_end) = cobs_buf[..read_pos].iter().position(|&b| b == 0x00) {
                // Found a COBS frame (ends with 0x00)
                let frame_data = &mut cobs_buf[..frame_end];
                let decoded_len = decode_in_place(frame_data).map_err(|e| anyhow!("COBS decode error: {:?}", e))?;

                match from_bytes::<McuToHost>(&frame_data[..decoded_len]) {
                    Ok(mcu_msg) => {
                        self.handle_mcu_message(mcu_msg).await?;
                    }
                    Err(e) => {
                        error!("Postcard deserialize error: {:?}", e);
                    }
                }

                // Shift remaining bytes
                let remaining_len = read_pos - (frame_end + 1);
                cobs_buf.copy_within(frame_end + 1..read_pos, 0);
                read_pos = remaining_len;
            }

            if read_pos >= cobs_buf.len() {
                warn!("COBS buffer overflow. Dropping data.");
                read_pos = 0; // Reset buffer
            }
        }
    }

    async fn write_loop(&mut self, writer: &mut (impl AsyncWriteExt + Unpin)) -> Result<()> {
        loop {
            let cmd = self.mcu_cmd_receiver.recv().await.ok_or_else(|| anyhow!("MCU command channel closed"))?;
            info!("Sending command to MCU: {:?}", cmd);

            let mut buf = [0u8; 256];
            let used = to_vec_cobs(&cmd, &mut buf).map_err(|e| anyhow!("Postcard serialize error: {:?}", e))?;

            match writer.write_all(used).await {
                Ok(_) => {
                    // Successfully sent
                }
                Err(e) => {
                    error!("Serial write error: {}", e);
                    return Err(anyhow!("Serial write error: {}", e));
                }
            }
        }
    }

    async fn handle_mcu_message(&self, msg: McuToHost) -> Result<()> {
        match msg {
            McuToHost::Telemetry(telemetry) => {
                let mut state = self.machine_state.write().await;
                state.nozzle_temp = telemetry.nozzle_temp;
                state.bed_temp = telemetry.bed_temp;
                // Update other machine state fields
                drop(state); // Release the write lock

                // Broadcast telemetry to WebSocket clients
                let json_telemetry = serde_json::to_value(&telemetry)?;
                if let Err(e) = self.telemetry_broadcaster.send(json_telemetry) {
                    error!("Failed to send telemetry to broadcaster: {}", e);
                }
            }
            McuToHost::Response(response) => {
                info!("Received MCU response: {:?}", response);
                // Handle specific responses, e.g., command acknowledgments
            }
            McuToHost::Error(e) => {
                error!("Received MCU error: {}", e);
                // Log or propagate error
            }
        }
        Ok(())
    }
}

// --- Tests ---
#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::DuplexStream;
    use tokio::time::timeout;

    // Helper to create a mock serial port
    fn create_mock_serial() -> (DuplexStream, DuplexStream) {
        tokio::io::duplex(256) // Max buffer size
    }

    #[tokio::test]
    async fn test_mcu_to_host_telemetry_serialization() -> Result<()> {
        let telemetry = Telemetry {
            nozzle_temp: 200.0,
            bed_temp: 60.0,
            x_pos: 10.0,
            y_pos: 20.0,
            z_pos: 5.0,
        };
        let mcu_msg = McuToHost::Telemetry(telemetry);

        let mut buf = [0u8; 256];
        let encoded = to_vec_cobs(&mcu_msg, &mut buf)?;

        let mut decoded_buf = [0u8; 256];
        decoded_buf[..encoded.len()].copy_from_slice(encoded);
        let decoded_len = decode_in_place(&mut decoded_buf[..encoded.len()])?;

        let decoded_msg: McuToHost = from_bytes(&decoded_buf[..decoded_len])?;

        assert_eq!(mcu_msg, decoded_msg);
        Ok(())
    }

    #[tokio::test]
    async fn test_host_to_mcu_gcode_serialization() -> Result<()> {
        let gcode = "G28 X Y".to_string();
        let host_msg = HostToMcu::GCode(gcode);

        let mut buf = [0u8; 256];
        let encoded = to_vec_cobs(&host_msg, &mut buf)?;

        let mut decoded_buf = [0u8; 256];
        decoded_buf[..encoded.len()].copy_from_slice(encoded);
        let decoded_len = decode_in_place(&mut decoded_buf[..encoded.len()])?;

        let decoded_msg: HostToMcu = from_bytes(&decoded_buf[..decoded_len])?;

        assert_eq!(host_msg, decoded_msg);
        Ok(())
    }

    #[tokio::test]
    async fn test_serial_bridge_read_telemetry() -> Result<()> {
        let (mut host_side_writer, mut mcu_side_reader) = create_mock_serial();
        let (tx, _rx) = broadcast::channel(10);
        let (_mcu_cmd_tx, mcu_cmd_rx) = mpsc::channel(10);
        let machine_state = Arc::new(RwLock::new(MachineState::default()));

        let bridge = SerialBridge::new(
            "/dev/ttyUSB_mock".to_string(),
            115200,
            tx.clone(),
            mcu_cmd_rx,
            machine_state.clone(),
        );

        let read_task = tokio::spawn(async move {
            bridge.read_loop(&mut mcu_side_reader).await
        });

        // Simulate MCU sending telemetry
        let telemetry = Telemetry {
            nozzle_temp: 200.0,
            bed_temp: 60.0,
            x_pos: 10.0,
            y_pos: 20.0,
            z_pos: 5.0,
        };
        let mcu_msg = McuToHost::Telemetry(telemetry.clone());
        let mut encoded_buf = [0u8; 256];
        let encoded_msg = to_vec_cobs(&mcu_msg, &mut encoded_buf)?;

        host_side_writer.write_all(encoded_msg).await?;
        host_side_writer.write_all(&[0x00]).await?; // COBS frame delimiter

        // Wait for machine state to be updated
        tokio::time::sleep(Duration::from_millis(100)).await;

        let state = machine_state.read().await;
        assert_eq!(state.nozzle_temp, telemetry.nozzle_temp);
        assert_eq!(state.bed_temp, telemetry.bed_temp);

        // Ensure telemetry was broadcasted
        let received_telemetry = timeout(Duration::from_millis(100), tx.subscribe().recv()).await??;
        assert_eq!(received_telemetry["nozzle_temp"], telemetry.nozzle_temp);

        // Stop the read task
        read_task.abort();
        Ok(())
    }

    #[tokio::test]
    async fn test_serial_bridge_write_gcode() -> Result<()> {
        let (mut host_side_reader, mut mcu_side_writer) = create_mock_serial();
        let (tx, _rx) = broadcast::channel(10);
        let (mcu_cmd_tx, mcu_cmd_rx) = mpsc::channel(10);
        let machine_state = Arc::new(RwLock::new(MachineState::default()));

        let mut bridge = SerialBridge::new(
            "/dev/ttyUSB_mock".to_string(),
            115200,
            tx.clone(),
            mcu_cmd_rx,
            machine_state.clone(),
        );

        let write_task = tokio::spawn(async move {
            bridge.write_loop(&mut mcu_side_writer).await
        });

        // Send a G-code command from host
        let gcode_cmd = "G1 X100".to_string();
        mcu_cmd_tx.send(HostToMcu::GCode(gcode_cmd.clone())).await?;

        // Read from the MCU side
        let mut received_bytes = Vec::new();
        let read_future = host_side_reader.read_to_end(&mut received_bytes);

        // Give some time for the write to happen, then abort the write task to unblock read_to_end
        tokio::time::sleep(Duration::from_millis(100)).await;
        write_task.abort();

        // Await the read_to_end, which will complete when the writer is dropped/aborted
        let _ = read_future.await;

        // COBS decode and deserialize
        let mut decoded_buf = received_bytes.clone();
        let decoded_len = decode_in_place(&mut decoded_buf)?;
        let decoded_msg: HostToMcu = from_bytes(&decoded_buf[..decoded_len])?;

        assert_eq!(decoded_msg, HostToMcu::GCode(gcode_cmd));
        Ok(())
    }
}
