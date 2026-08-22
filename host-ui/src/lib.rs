slint::include_modules!();

use anyhow::Result;
use slint::Model;
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::rc::Rc;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

// --- Commands sent from UI to MCU ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HostToMcu {
    GCode(String),
}

const API_BASE_URL: &str = "http://127.0.0.1:7125/api";
const WS_URL: &str = "ws://127.0.0.1:7125/websocket";

pub async fn run_ui(mcu_cmd_sender: mpsc::Sender<HostToMcu>) -> Result<()> {
    let ui = AppWindow::new()?;

    let http_client = Client::new();
    let ui_handle = ui.as_weak();

    // --- Telemetry WebSocket Client ---
    let ui_handle_clone = ui_handle.clone();
    tokio::spawn(async move {
        info!("Connecting to WebSocket: {}", WS_URL);
        let url = match Url::parse(WS_URL) {
            Ok(u) => u,
            Err(e) => {
                error!("Invalid websocket URL: {}", e);
                return;
            }
        };

        loop {
            match connect_async(url.clone()).await {
                Ok((ws_stream, _)) => {
                    info!("WebSocket connected.");
                    let (mut write, mut read) = ws_stream.split();

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
                                if let Ok(telemetry) = serde_json::from_str::<serde_json::Value>(&text) {
                                    let ui_clone = ui_handle_clone.clone();
                                    let _ = slint::invoke_from_event_loop(move || {
                                        if let Some(ui) = ui_clone.upgrade() {
                                            if let Some(nozzle_temp) = telemetry["nozzle_temp"].as_f64() {
                                                ui.set_nozzle_temp_actual(nozzle_temp as f32);
                                            }
                                            if let Some(bed_temp) = telemetry["bed_temp"].as_f64() {
                                                ui.set_bed_temp_actual(bed_temp as f32);
                                            }
                                        }
                                    });
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
                            _ => {}
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

    // --- Callbacks ---
    let http_client_clone = http_client.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_set_nozzle_target(move |temp| {
        let client = http_client_clone.clone();
        let ui_handle = ui_handle_clone.clone();
        tokio::spawn(async move {
            info!("Setting nozzle target to {}°C", temp);
            let body = serde_json::json!({
                "method": "printer.toolhead.set_temperature",
                "params": { "temperature": temp }
            });
            if let Ok(resp) = client.post(format!("{}/rpc", API_BASE_URL)).json(&body).send().await {
                if resp.status().is_success() {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_nozzle_temp_target(temp);
                        }
                    });
                }
            }
        });
    });

    let http_client_clone = http_client.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_set_bed_target(move |temp| {
        let client = http_client_clone.clone();
        let ui_handle = ui_handle_clone.clone();
        tokio::spawn(async move {
            info!("Setting bed target to {}°C", temp);
            let body = serde_json::json!({
                "method": "printer.bed.set_temperature",
                "params": { "temperature": temp }
            });
            if let Ok(resp) = client.post(format!("{}/rpc", API_BASE_URL)).json(&body).send().await {
                if resp.status().is_success() {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_bed_temp_target(temp);
                        }
                    });
                }
            }
        });
    });

    let http_client_clone = http_client.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_refresh_files(move || {
        let client = http_client_clone.clone();
        let ui_handle = ui_handle_clone.clone();
        tokio::spawn(async move {
            info!("Refreshing G-code files...");
            if let Ok(resp) = client.get(format!("{}/files", API_BASE_URL)).send().await {
                if resp.status().is_success() {
                    if let Ok(json_resp) = resp.json::<serde_json::Value>().await {
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

                            let _ = slint::invoke_from_event_loop(move || {
                                if let Some(ui) = ui_handle.upgrade() {
                                    ui.set_gcode_files(Rc::new(slint::VecModel::from(gcode_files)).into());
                                }
                            });
                        }
                    }
                }
            }
        });
    });

    let http_client_clone = http_client.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_start_print(move |file_name| {
        let client = http_client_clone.clone();
        let ui_handle = ui_handle_clone.clone();
        let file_name_str = file_name.to_string();
        tokio::spawn(async move {
            info!("Starting print for file: {}", file_name_str);
            if let Ok(resp) = client.post(format!("{}/print/start/{}", API_BASE_URL, file_name_str)).send().await {
                if resp.status().is_success() {
                    let _ = slint::invoke_from_event_loop(move || {
                        if let Some(ui) = ui_handle.upgrade() {
                            ui.set_current_print_file(file_name_str.as_str().into());
                            ui.set_print_progress(0.0);
                        }
                    });
                }
            }
        });
    });

    let mcu_cmd_sender_clone = mcu_cmd_sender.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_send_gcode_command(move |command| {
        let sender = mcu_cmd_sender_clone.clone();
        let ui_handle = ui_handle_clone.clone();
        let command_str = command.to_string();
        tokio::spawn(async move {
            info!("Sending G-code command: {}", command_str);
            let _ = sender.send(HostToMcu::GCode(command_str.clone())).await;
            let _ = slint::invoke_from_event_loop(move || {
                if let Some(ui) = ui_handle.upgrade() {
                    let mut console = ui.get_console_output().iter().collect::<Vec<_>>();
                    console.push(format!("> {}", command_str).into());
                    ui.set_console_output(Rc::new(slint::VecModel::from(console)).into());
                }
            });
        });
    });

    ui.run()?;

    Ok(())
}
