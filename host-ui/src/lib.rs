slint::include_modules!();

use anyhow::Result;
use futures_util::{SinkExt, StreamExt};
use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use slint::Model;
use std::rc::Rc;
use tokio::sync::mpsc;
use tokio::time::Duration;
use tokio_tungstenite::{connect_async, tungstenite::Message};
use url::Url;

// --- Commands sent from UI to MCU / Host ---
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum HostToMcu {
    GCode(String),
}

const API_BASE_URL: &str = "http://127.0.0.1:7125/api";
const WS_URL: &str = "ws://127.0.0.1:7125/websocket";

/// Printer state struct holding full printer telemetry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrinterState {
    pub printer_state: String,
    pub extruder_temp: f32,
    pub extruder_target: f32,
    pub bed_temp: f32,
    pub bed_target: f32,
    pub fan_speed: f32,
    pub print_progress: f32,
    pub print_filename: String,
    pub raw_state: Value,
}

impl PrinterState {
    pub fn new() -> Self {
        Self {
            printer_state: "idle".to_string(),
            extruder_temp: 0.0,
            extruder_target: 0.0,
            bed_temp: 0.0,
            bed_target: 0.0,
            fan_speed: 0.0,
            print_progress: 0.0,
            print_filename: String::new(),
            raw_state: serde_json::json!({
                "interpreter_state": "idle",
                "temperature_devices": { "count": 1 },
                "extruders": { "count": 1 },
                "fans": { "count": 1 },
            }),
        }
    }
}

pub fn merge_json(dest: &mut Value, src: &Value) {
    if let (Some(dest_obj), Some(src_obj)) = (dest.as_object_mut(), src.as_object()) {
        for (key, val) in src_obj {
            if val.is_object() {
                if !dest_obj.contains_key(key) {
                    dest_obj.insert(key.clone(), Value::Object(serde_json::Map::new()));
                }
                merge_json(&mut dest_obj[key], val);
            } else {
                dest_obj.insert(key.clone(), val.clone());
            }
        }
    }
}

pub fn reduce_printer_state(state: &mut PrinterState, update: &Value) {
    merge_json(&mut state.raw_state, update);

    if let Some(obj) = update.as_object() {
        if let Some(extruder) = obj.get("extruder") {
            if let Some(temp) = extruder.get("temperature").and_then(|t| t.as_f64()) {
                state.extruder_temp = temp as f32;
            }
            if let Some(target) = extruder.get("target").and_then(|t| t.as_f64()) {
                state.extruder_target = target as f32;
            }
        }
        if let Some(bed) = obj.get("heater_bed") {
            if let Some(temp) = bed.get("temperature").and_then(|t| t.as_f64()) {
                state.bed_temp = temp as f32;
            }
            if let Some(target) = bed.get("target").and_then(|t| t.as_f64()) {
                state.bed_target = target as f32;
            }
        }
        if let Some(fan) = obj.get("fan") {
            if let Some(speed) = fan.get("speed").and_then(|s| s.as_f64()) {
                state.fan_speed = speed as f32;
            }
        }
        if let Some(stats) = obj.get("print_stats") {
            if let Some(progress) = stats.get("progress").and_then(|p| p.as_f64()) {
                state.print_progress = (progress * 100.0) as f32;
            }
            if let Some(filename) = stats.get("filename").and_then(|f| f.as_str()) {
                state.print_filename = filename.to_string();
            }
            if let Some(print_state) = stats.get("state").and_then(|s| s.as_str()) {
                state.printer_state = print_state.to_string();
            }
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
        let ws_url = match Url::parse(WS_URL) {
            Ok(u) => u,
            Err(e) => {
                error!("Invalid websocket URL: {}", e);
                return;
            }
        };

        loop {
            match connect_async(ws_url.as_str()).await {
                Ok((ws_stream, _)) => {
                    info!("WebSocket connected.");
                    let (mut write, mut read) = ws_stream.split();

                    // Send Moonraker object subscription
                    let subscribe_msg = serde_json::json!({
                        "jsonrpc": "2.0",
                        "method": "printer.objects.subscribe",
                        "params": {
                            "objects": {
                                "extruder": ["temperature", "target"],
                                "heater_bed": ["temperature", "target"],
                                "fan": ["speed"],
                                "print_stats": ["filename", "progress", "state"]
                            }
                        },
                        "id": 1
                    });
                    let _ = write.send(Message::Text(subscribe_msg.to_string().into())).await;

                    let mut ping_interval = tokio::time::interval(Duration::from_secs(5));
                    let write_handle = tokio::spawn(async move {
                        loop {
                            ping_interval.tick().await;
                            if write.send(Message::Ping(Default::default())).await.is_err() {
                                error!("WebSocket write error, reconnecting...");
                                break;
                            }
                        }
                    });

                    let mut local_state = PrinterState::new();

                    while let Some(message) = read.next().await {
                        match message {
                            Ok(Message::Text(text)) => {
                                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                                    if json["method"].as_str() == Some("notify_status_update") {
                                        if let Some(params) = json["params"].get(0) {
                                            reduce_printer_state(&mut local_state, params);
                                            let ext_temp = local_state.extruder_temp;
                                            let ext_tgt = local_state.extruder_target;
                                            let bed_temp = local_state.bed_temp;
                                            let bed_tgt = local_state.bed_target;
                                            let fan_spd = local_state.fan_speed;
                                            let prog = local_state.print_progress;
                                            let pstate = local_state.printer_state.clone();
                                            let fname = local_state.print_filename.clone();

                                            let ui_c = ui_handle_clone.clone();
                                            let _ = slint::invoke_from_event_loop(move || {
                                                if let Some(ui) = ui_c.upgrade() {
                                                    ui.set_nozzle_temp_actual(ext_temp);
                                                    ui.set_nozzle_temp_target(ext_tgt);
                                                    ui.set_bed_temp_actual(bed_temp);
                                                    ui.set_bed_temp_target(bed_tgt);
                                                    ui.set_fan_speed(fan_spd);
                                                    ui.set_print_progress(prog);
                                                    ui.set_printer_state(slint::SharedString::from(pstate));
                                                    ui.set_current_print_file(slint::SharedString::from(fname));
                                                }
                                            });
                                        }
                                    }
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
        tokio::spawn(async move {
            let _ = client
                .post(format!("{}/printer/extruder/target", API_BASE_URL))
                .json(&serde_json::json!({ "target": temp }))
                .send()
                .await;
        });
        if let Some(ui) = ui_handle_clone.upgrade() {
            ui.set_nozzle_temp_target(temp);
        }
    });

    let http_client_clone = http_client.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_set_bed_target(move |temp| {
        let client = http_client_clone.clone();
        tokio::spawn(async move {
            let _ = client
                .post(format!("{}/printer/bed/target", API_BASE_URL))
                .json(&serde_json::json!({ "target": temp }))
                .send()
                .await;
        });
        if let Some(ui) = ui_handle_clone.upgrade() {
            ui.set_bed_temp_target(temp);
        }
    });

    let http_client_clone = http_client.clone();
    let mcu_sender_clone = mcu_cmd_sender.clone();
    ui.on_start_print(move |file_name| {
        let client = http_client_clone.clone();
        let mcu_tx = mcu_sender_clone.clone();
        let name_str = file_name.to_string();
        tokio::spawn(async move {
            let _ = client
                .post(format!("{}/printer/print/start", API_BASE_URL))
                .json(&serde_json::json!({ "filename": name_str }))
                .send()
                .await;
            let _ = mcu_tx.send(HostToMcu::GCode(format!("SDCARD_PRINT_FILE FILENAME=\"{}\"", name_str))).await;
        });
    });

    let http_client_clone = http_client.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_refresh_files(move || {
        let client = http_client_clone.clone();
        let ui_c = ui_handle_clone.clone();
        tokio::spawn(async move {
            if let Ok(res) = client.get(format!("{}/server/files/list", API_BASE_URL)).send().await {
                if let Ok(json) = res.json::<serde_json::Value>().await {
                    if let Some(files_array) = json.as_array() {
                        let gcode_files: Vec<GCodeFileInfo> = files_array
                            .iter()
                            .filter_map(|f| {
                                let name = f["name"].as_str()?.to_string();
                                let size = f["size"].as_str().unwrap_or("0 B").to_string();
                                let is_dir = f["is_dir"].as_bool().unwrap_or(false);
                                Some(GCodeFileInfo {
                                    name: slint::SharedString::from(name),
                                    size: slint::SharedString::from(size),
                                    is_dir,
                                    thumbnail: Default::default(),
                                })
                            })
                            .collect();
                        let _ = slint::invoke_from_event_loop(move || {
                            if let Some(ui) = ui_c.upgrade() {
                                ui.set_gcode_files(Rc::new(slint::VecModel::from(gcode_files)).into());
                            }
                        });
                    }
                }
            }
        });
    });

    let mcu_sender_clone = mcu_cmd_sender.clone();
    let ui_handle_clone = ui_handle.clone();
    ui.on_send_gcode_command(move |command| {
        let cmd = command.to_string();
        let mcu_tx = mcu_sender_clone.clone();
        let ui_c = ui_handle_clone.clone();
        tokio::spawn(async move {
            let _ = mcu_tx.send(HostToMcu::GCode(cmd.clone())).await;
        });
        if let Some(ui) = ui_c.upgrade() {
            let mut lines: Vec<ConsoleLine> = ui.get_console_lines().iter().collect();
            lines.push(ConsoleLine {
                text: slint::SharedString::from(format!("> {}", command)),
                is_input: true,
            });
            ui.set_console_lines(Rc::new(slint::VecModel::from(lines)).into());
        }
    });

    let mcu_sender_clone = mcu_cmd_sender.clone();
    ui.on_emergency_stop(move || {
        let tx = mcu_sender_clone.clone();
        tokio::spawn(async move {
            let _ = tx.send(HostToMcu::GCode("M112".to_string())).await;
        });
    });

    let mcu_sender_clone = mcu_cmd_sender.clone();
    ui.on_home_axis(move |axis| {
        let tx = mcu_sender_clone.clone();
        let ax = axis.to_string();
        tokio::spawn(async move {
            let cmd = if ax == "all" || ax == "ALL" { "G28".to_string() } else { format!("G28 {}", ax) };
            let _ = tx.send(HostToMcu::GCode(cmd)).await;
        });
    });

    let mcu_sender_clone = mcu_cmd_sender.clone();
    ui.on_jog_axis(move |axis, dist, spd| {
        let tx = mcu_sender_clone.clone();
        let ax = axis.to_string();
        tokio::spawn(async move {
            let _ = tx.send(HostToMcu::GCode("G91".to_string())).await;
            let _ = tx.send(HostToMcu::GCode(format!("G1 {}{} F{}", ax, dist, spd * 60.0))).await;
            let _ = tx.send(HostToMcu::GCode("G90".to_string())).await;
        });
    });

    let mcu_sender_clone = mcu_cmd_sender.clone();
    ui.on_disable_motors(move || {
        let tx = mcu_sender_clone.clone();
        tokio::spawn(async move {
            let _ = tx.send(HostToMcu::GCode("M84".to_string())).await;
        });
    });

    let mcu_sender_clone = mcu_cmd_sender.clone();
    ui.on_extrude(move |dist, spd| {
        let tx = mcu_sender_clone.clone();
        tokio::spawn(async move {
            let _ = tx.send(HostToMcu::GCode("M83".to_string())).await;
            let _ = tx.send(HostToMcu::GCode(format!("G1 E{} F{}", dist, spd * 60.0))).await;
        });
    });

    let mcu_sender_clone = mcu_cmd_sender.clone();
    ui.on_set_fan_speed(move |_idx, spd| {
        let tx = mcu_sender_clone.clone();
        let pwm = (spd * 255.0) as u32;
        tokio::spawn(async move {
            let _ = tx.send(HostToMcu::GCode(format!("M106 S{}", pwm))).await;
        });
    });

    let mcu_sender_clone = mcu_cmd_sender.clone();
    ui.on_pause_print(move || {
        let tx = mcu_sender_clone.clone();
        tokio::spawn(async move {
            let _ = tx.send(HostToMcu::GCode("PAUSE".to_string())).await;
        });
    });

    let mcu_sender_clone = mcu_cmd_sender.clone();
    ui.on_resume_print(move || {
        let tx = mcu_sender_clone.clone();
        tokio::spawn(async move {
            let _ = tx.send(HostToMcu::GCode("RESUME".to_string())).await;
        });
    });

    let mcu_sender_clone = mcu_cmd_sender.clone();
    ui.on_cancel_print(move || {
        let tx = mcu_sender_clone.clone();
        tokio::spawn(async move {
            let _ = tx.send(HostToMcu::GCode("CANCEL_PRINT".to_string())).await;
        });
    });

    let mcu_sender_clone = mcu_cmd_sender.clone();
    ui.on_run_macro(move |name| {
        let tx = mcu_sender_clone.clone();
        let macro_name = name.to_string();
        tokio::spawn(async move {
            let _ = tx.send(HostToMcu::GCode(macro_name)).await;
        });
    });

    ui.run()?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_printer_state_reducer() {
        let mut state = PrinterState::new();
        let update = json!({
            "extruder": {
                "temperature": 215.5,
                "target": 220.0
            },
            "heater_bed": {
                "temperature": 60.2,
                "target": 65.0
            },
            "fan": {
                "speed": 0.75
            },
            "print_stats": {
                "progress": 0.42,
                "filename": "voron_cube.gcode",
                "state": "printing"
            }
        });

        reduce_printer_state(&mut state, &update);

        assert_eq!(state.extruder_temp, 215.5);
        assert_eq!(state.extruder_target, 220.0);
        assert_eq!(state.bed_temp, 60.2);
        assert_eq!(state.bed_target, 65.0);
        assert_eq!(state.fan_speed, 0.75);
        assert_eq!(state.print_progress, 42.0);
        assert_eq!(state.print_filename, "voron_cube.gcode");
        assert_eq!(state.printer_state, "printing");
    }

    #[test]
    fn test_merge_json() {
        let mut dest = json!({
            "key1": "val1",
            "nested": {
                "inner1": "old"
            }
        });
        let src = json!({
            "key2": "val2",
            "nested": {
                "inner2": "new"
            }
        });

        merge_json(&mut dest, &src);

        assert_eq!(dest["key1"], "val1");
        assert_eq!(dest["key2"], "val2");
        assert_eq!(dest["nested"]["inner1"], "old");
        assert_eq!(dest["nested"]["inner2"], "new");
    }
}
