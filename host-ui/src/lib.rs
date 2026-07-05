slint::include_modules!();

use anyhow::{anyhow, Result};
use futures_util::{SinkExt, StreamExt};
use log::{error, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, Mutex};
use tokio::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

use host_server::bridge::HostToMcu; // Import HostToMcu from host-server

const API_BASE_URL: &str = "http://127.0.0.1:7125/api";
const WS_URL: &str = "ws://127.0.0.1:7125/websocket";

// Struct to match the GCodeFile from the Slint UI
#[derive(Default, Clone, Debug, slint::SharedString, Serialize, Deserialize)]
pub struct GCodeFile {
    pub name: slint::SharedString,
    pub size: slint::SharedString,
    pub upload_date: slint::SharedString,
}

// Implement the `Model` trait for `GCodeFile` so it can be used in `ListView`
impl slint::Model for GCodeFile {
    type Data = GCodeFile;
    fn row_count(&self) -> usize {
        1
    }
    fn row_data(&self, row: usize) -> Option<Self::Data> {
        if row == 0 {
            Some(self.clone())
        } else {
            None
        }
    }
}

pub async fn run_ui(mcu_cmd_sender: mpsc::Sender<HostToMcu>) -> Result<()> {
    let ui = AppWindow::new()?;

    let http_client = Client::new();
    let ui_handle = ui.as_weak();

    // --- Telemetry WebSocket Client ---
    let ui_handle_clone = ui_handle.clone();
    tokio::spawn(async move {
        info!("Connecting to WebSocket: {}", WS_URL);
        let url = Url::parse(WS_URL).expect("Can't parse websocket URL");

        loop {
            match connect_async(url.clone()).await {
                Ok((ws_stream, _)) => {
                    info!("WebSocket connected.");
                    let (mut write, mut read) = ws_stream.split();

                    // Ping every 5 seconds to keep connection alive
                    let mut ping_interval = tokio::time::interval(Duration::from_secs(5));
                    let write_handle = tokio::spawn(async move {
                        loop {
                            ping_interval.tick().await;
                            if write.send(Message::Ping(vec![])).await.is_err() {
                                error!("WebSocket write error, reconnecting...");
                                break;
                            }
                        }
                    });

                    while let Some(message) = read.next().await {
                        match message {
                            Ok(Message::Text(text)) => {
                                // info!("Received telemetry: {}", text);
                                if let Ok(telemetry) = serde_json::from_str::<serde_json::Value>(&text) {
                                    if let Some(nozzle_temp) = telemetry["nozzle_temp"].as_f64() {
                                        slint::invoke_from_event_loop(move || {
                                            if let Some(ui) = ui_handle_clone.upgrade() {
                                                ui.set_nozzle_temp_actual(nozzle_temp as f32);
                                            }
                                        })
                                        .unwrap();
                                    }
                                    if let Some(bed_temp) = telemetry["bed_temp"].as_f64() {
                                        slint::invoke_from_event_loop(move || {
                                            if let Some(ui) = ui_handle_clone.upgrade() {
                                                ui.set_bed_temp_actual(bed_temp as f32);
                                            }
                                        })
                                        .unwrap();
                                    }
                                    // Update other UI elements based on telemetry
                                }
                            }
                            Ok(Message::Close(_)) => {
                                info!("WebSocket closed gracefully.");
                                break;
                            }
                            Err(e) => {
                                error!("WebSocket error: {}", e);
                                break;
                            }
                            _ => {} // Ignore other message types like Ping/Pong/Binary
                        }
                    }
                    write_handle.abort();
                }
                Err(e) => {
                    error!("Failed to connect to WebSocket: {}. Retrying in 3 seconds...", e);
                }
            }
            tokio::time::sleep(Duration::from_secs(3)).await;
        }
    });

    // --- Callbacks to Actix-Web API ---
    let http_client_clone = http_client.clone();
    ui.on_set_nozzle_target(move |temp| {
        let client = http_client_clone.clone();
        let ui_handle_clone = ui_handle.clone();
        tokio::spawn(async move {
            info!("Setting nozzle target to {}°C", temp);
            let body = serde_json::json!({
                "method": "printer.toolhead.set_temperature",
                "params": { "temperature": temp }
            });
            match client.post(format!("{}/rpc", API_BASE_URL)).json(&body).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        info!("Nozzle target set successfully.");
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_handle_clone.upgrade() {
                                ui.set_nozzle_temp_target(temp);
                            }
                        }).unwrap();
                    } else {
                        error!("Failed to set nozzle target: {:?}", resp.text().await);
                    }
                }
                Err(e) => error!("HTTP request failed: {}", e),
            }
        });
    });

    let http_client_clone = http_client.clone();
    ui.on_set_bed_target(move |temp| {
        let client = http_client_clone.clone();
        let ui_handle_clone = ui_handle.clone();
        tokio::spawn(async move {
            info!("Setting bed target to {}°C", temp);
            let body = serde_json::json!({
                "method": "printer.bed.set_temperature",
                "params": { "temperature": temp }
            });
            match client.post(format!("{}/rpc", API_BASE_URL)).json(&body).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        info!("Bed target set successfully.");
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_handle_clone.upgrade() {
                                ui.set_bed_temp_target(temp);
                            }
                        }).unwrap();
                    } else {
                        error!("Failed to set bed target: {:?}", resp.text().await);
                    }
                }
                Err(e) => error!("HTTP request failed: {}", e),
            }
        });
    });

    let http_client_clone = http_client.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_refresh_files(move || {
        let client = http_client_clone.clone();
        let ui_handle_clone = ui_handle_clone.clone();
        tokio::spawn(async move {
            info!("Refreshing G-code files...");
            match client.get(format!("{}/files", API_BASE_URL)).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        let json_resp: serde_json::Value = resp.json().await?;
                        if let Some(files_array) = json_resp["files"].as_array() {
                            let gcode_files: Vec<GCodeFile> = files_array
                                .iter()
                                .filter_map(|f| {
                                    let path = f["path"].as_str()?;
                                    let name = path.split('/').last().unwrap_or(path);
                                    let size = f["size"].as_u64()?;
                                    let upload_date = f["upload_date"].as_str()?;
                                    Some(GCodeFile {
                                        name: name.into(),
                                        size: format!("{:.2} MB", size as f64 / 1_000_000.0).into(),
                                        upload_date: upload_date.split('T').next().unwrap_or(upload_date).into(),
                                    })
                                })
                                .collect();

                            slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_handle_clone.upgrade() {
                                    ui.set_gcode_files(Rc::new(slint::VecModel::from(gcode_files)));
                                }
                            }).unwrap();
                        }
                    } else {
                        error!("Failed to refresh files: {:?}", resp.text().await);
                    }
                }
                Err(e) => error!("HTTP request failed: {}", e),
            }
            Ok::<(), anyhow::Error>(())
        });
    });

    let http_client_clone = http_client.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_start_print(move |file_name| {
        let client = http_client_clone.clone();
        let ui_handle_clone = ui_handle_clone.clone();
        let file_name_str = file_name.to_string();
        tokio::spawn(async move {
            info!("Starting print for file: {}", file_name_str);
            match client.post(format!("{}/print/start/{}", API_BASE_URL, file_name_str)).send().await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        info!("Print started successfully.");
                        slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_handle_clone.upgrade() {
                                ui.set_current_print_file(file_name_str.into());
                                ui.set_print_progress(0.0);
                                // Add to console output
                                let mut console = ui.get_console_output().to_vec();
                                console.push(format!("Print started: {}", file_name_str).into());
                                ui.set_console_output(Rc::new(slint::VecModel::from(console)));
                            }
                        }).unwrap();
                    } else {
                        error!("Failed to start print: {:?}", resp.text().await);
                    }
                }
                Err(e) => error!("HTTP request failed: {}", e),
            }
        });
    });

    let mcu_cmd_sender_clone = mcu_cmd_sender.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_send_gcode_command(move |command| {
        let mut sender = mcu_cmd_sender_clone.clone();
        let ui_handle_clone = ui_handle_clone.clone();
        let command_str = command.to_string();
        tokio::spawn(async move {
            info!("Sending G-code command to MCU: {}", command_str);
            if let Err(e) = sender.send(HostToMcu::GCode(command_str.clone())).await {
                error!("Failed to send G-code to MCU via channel: {}", e);
            } else {
                slint::invoke_from_event_loop(move || {
                    if let Some(ui) = ui_handle_clone.upgrade() {
                        let mut console = ui.get_console_output().to_vec();
                        console.push(format!("> {}", command_str).into());
                        ui.set_console_output(Rc::new(slint::VecModel::from(console)));
                    }
                }).unwrap();
            }
        });
    });

    ui.run()?;

    Ok(())
}
