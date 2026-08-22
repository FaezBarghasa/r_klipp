use anyhow::{anyhow, Result};
use cobs::decode_in_place;
use log::{error, info, warn};
use postcard::{from_bytes, to_allocvec_cobs};
use serde::{Deserialize, Serialize};
use std::io::ErrorKind;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::{broadcast, mpsc, RwLock};
use tokio::time::sleep;
use tokio_serial::{SerialPortBuilderExt, SerialStream};

use crate::api::MachineState;

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
                Ok(port) => {
                    info!("Connected to serial port: {}", self.port_path);
                    let (mut reader, mut writer) = tokio::io::split(port);

                    let telemetry_tx = self.telemetry_broadcaster.clone();
                    let state = self.machine_state.clone();

                    let read_loop = Self::read_loop(telemetry_tx, state, &mut reader);
                    let write_loop = Self::write_loop(&mut self.mcu_cmd_receiver, &mut writer);

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

    async fn read_loop(
        telemetry_broadcaster: broadcast::Sender<serde_json::Value>,
        machine_state: Arc<RwLock<MachineState>>,
        reader: &mut (impl AsyncReadExt + Unpin),
    ) -> Result<()> {
        let mut buf = vec![0u8; 256];
        let mut cobs_buf = vec![0u8; 256];
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

            while let Some(frame_end) = cobs_buf[..read_pos].iter().position(|&b| b == 0x00) {
                let frame_data = &mut cobs_buf[..frame_end];
                let decoded_len = decode_in_place(frame_data).map_err(|e| anyhow!("COBS decode error: {:?}", e))?;

                match from_bytes::<McuToHost>(&frame_data[..decoded_len]) {
                    Ok(mcu_msg) => {
                        Self::handle_mcu_message(&telemetry_broadcaster, &machine_state, mcu_msg).await?;
                    }
                    Err(e) => {
                        error!("Postcard deserialize error: {:?}", e);
                    }
                }

                let remaining_len = read_pos - (frame_end + 1);
                cobs_buf.copy_within(frame_end + 1..read_pos, 0);
                read_pos = remaining_len;
            }

            if read_pos >= cobs_buf.len() {
                warn!("COBS buffer overflow. Dropping data.");
                read_pos = 0;
            }
        }
    }

    async fn write_loop(cmd_rx: &mut mpsc::Receiver<HostToMcu>, writer: &mut (impl AsyncWriteExt + Unpin)) -> Result<()> {
        loop {
            let cmd = cmd_rx.recv().await.ok_or_else(|| anyhow!("MCU command channel closed"))?;
            info!("Sending command to MCU: {:?}", cmd);

            let used = to_allocvec_cobs(&cmd).map_err(|e| anyhow!("Postcard serialize error: {:?}", e))?;

            match writer.write_all(&used).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Serial write error: {}", e);
                    return Err(anyhow!("Serial write error: {}", e));
                }
            }
        }
    }

    async fn handle_mcu_message(
        telemetry_broadcaster: &broadcast::Sender<serde_json::Value>,
        machine_state: &Arc<RwLock<MachineState>>,
        msg: McuToHost,
    ) -> Result<()> {
        match msg {
            McuToHost::Telemetry(telemetry) => {
                let mut state = machine_state.write().await;
                state.nozzle_temp = telemetry.nozzle_temp;
                state.bed_temp = telemetry.bed_temp;
                drop(state);

                let json_telemetry = serde_json::to_value(&telemetry)?;
                if let Err(e) = telemetry_broadcaster.send(json_telemetry) {
                    error!("Failed to send telemetry to broadcaster: {}", e);
                }
            }
            McuToHost::Response(response) => {
                info!("Received MCU response: {:?}", response);
            }
            McuToHost::Error(e) => {
                error!("Received MCU error: {}", e);
            }
        }
        Ok(())
    }
}
