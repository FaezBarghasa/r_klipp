//! Moonraker-compatible HTTP & WebSocket API endpoints for Fluidd/Mainsail.

use actix_web::{web, App, HttpServer, HttpResponse, Responder};
use actix_cors::Cors;
use actix_ws::{Message, ProtocolError};
use futures_util::{StreamExt, SinkExt};
use tokio::sync::{broadcast, mpsc, RwLock};
use std::{sync::Arc, time::Duration};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use serde_json::json;
use log::{info, error};

use crate::db::{Database, HostError};
use crate::db::models::{GCodeFile, GCodeMetadata, PrintHistory, PrintStatus};
use crate::bridge::HostToMcu;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolheadState {
    pub position: [f64; 4], // X, Y, Z, E
    pub max_velocity: f64,
    pub max_accel: f64,
    pub homed_axes: String,
}

impl Default for ToolheadState {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0, 0.0],
            max_velocity: 300.0,
            max_accel: 3000.0,
            homed_axes: "xyz".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MachineState {
    pub nozzle_temp: f32,
    pub nozzle_target: f32,
    pub bed_temp: f32,
    pub bed_target: f32,
    pub toolhead: ToolheadState,
    pub current_print_file: Option<String>,
    pub print_progress: f32,
    pub state_message: String,
    pub is_printing: bool,
}

impl Default for MachineState {
    fn default() -> Self {
        Self {
            nozzle_temp: 22.0,
            nozzle_target: 0.0,
            bed_temp: 22.0,
            bed_target: 0.0,
            toolhead: ToolheadState::default(),
            current_print_file: None,
            print_progress: 0.0,
            state_message: "Printer is ready".to_string(),
            is_printing: false,
        }
    }
}

pub struct AppState {
    pub db: Arc<Database>,
    pub telemetry_broadcaster: broadcast::Sender<serde_json::Value>,
    pub mcu_cmd_sender: mpsc::Sender<HostToMcu>,
    pub machine_state: Arc<RwLock<MachineState>>,
}

// ------------------------------------------------------------------------------------------------
// Moonraker Core API Endpoints
// ------------------------------------------------------------------------------------------------

/// GET /printer/info -> Basic printer status
pub async fn get_printer_info(state: web::Data<AppState>) -> HttpResponse {
    let machine_state = state.machine_state.read().await;
    HttpResponse::Ok().json(json!({
        "result": {
            "state": if machine_state.is_printing { "printing" } else { "ready" },
            "state_message": machine_state.state_message,
            "hostname": "r-klipp-host",
            "klipper_path": "/home/jrad/r_klipp",
            "python_path": "/usr/bin/python3",
            "log_file": "/tmp/r_klipp.log",
            "config_file": "/home/jrad/printer.cfg",
            "software_version": "0.1.0-rklipp"
        }
    }))
}

/// Query parameters for /printer/objects/query
#[derive(Deserialize)]
pub struct ObjectQueryParams {
    pub objects: Option<String>,
}

/// GET /printer/objects/query -> Query hardware objects state (heaters, toolhead, print_stats)
pub async fn query_printer_objects(
    state: web::Data<AppState>,
    _query: web::Query<ObjectQueryParams>,
) -> HttpResponse {
    let ms = state.machine_state.read().await;
    HttpResponse::Ok().json(json!({
        "result": {
            "status": {
                "webhooks": {
                    "state": if ms.is_printing { "printing" } else { "ready" },
                    "state_message": ms.state_message
                },
                "extruder": {
                    "temperature": ms.nozzle_temp,
                    "target": ms.nozzle_target,
                    "power": 0.0,
                    "can_extrude": true
                },
                "heater_bed": {
                    "temperature": ms.bed_temp,
                    "target": ms.bed_target,
                    "power": 0.0
                },
                "toolhead": {
                    "position": ms.toolhead.position,
                    "max_velocity": ms.toolhead.max_velocity,
                    "max_accel": ms.toolhead.max_accel,
                    "homed_axes": ms.toolhead.homed_axes
                },
                "print_stats": {
                    "filename": ms.current_print_file,
                    "progress": ms.print_progress,
                    "state": if ms.is_printing { "printing" } else { "standby" }
                }
            }
        }
    }))
}

#[derive(Deserialize)]
pub struct GcodeScriptReq {
    pub script: String,
}

/// POST /printer/gcode/script -> Execute arbitrary G-Code script
pub async fn post_gcode_script(
    req: web::Json<GcodeScriptReq>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let script = req.script.trim();
    info!("Executing G-Code script: {}", script);

    for line in script.lines() {
        let line = line.trim();
        if !line.is_empty() && !line.starts_with(';') {
            let _ = state.mcu_cmd_sender.send(HostToMcu::GCode(line.to_string())).await;
        }
    }

    HttpResponse::Ok().json(json!({ "result": "ok" }))
}

/// GET /server/info -> Moonraker Server Info
pub async fn get_server_info() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "result": {
            "klippy_state": "ready",
            "klippy_connected": true,
            "api_version": [0, 1, 0],
            "api_version_string": "0.1.0-rklipp",
            "hostname": "r-klipp-host",
            "plugins": ["database", "file_manager", "gcode"]
        }
    }))
}

/// GET /server/files/list -> List stored G-Code files
pub async fn get_server_files_list(state: web::Data<AppState>) -> HttpResponse {
    match state.db.get_gcode_files().await {
        Ok(files) => HttpResponse::Ok().json(json!({ "result": files })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

// ------------------------------------------------------------------------------------------------
// WebSocket Stream
// ------------------------------------------------------------------------------------------------

pub async fn websocket_route(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    let mut rx = state.telemetry_broadcaster.subscribe();

    actix_web::rt::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));

        loop {
            tokio::select! {
                msg = msg_stream.next() => {
                    match msg {
                        Some(Ok(Message::Ping(bytes))) => {
                            if session.pong(&bytes).await.is_err() { break; }
                        }
                        Some(Ok(Message::Text(text))) => {
                            info!("WebSocket client message: {}", text);
                        }
                        Some(Ok(Message::Close(_))) => break,
                        _ => break,
                    }
                }
                telemetry = rx.recv() => {
                    if let Ok(data) = telemetry {
                        if session.text(data.to_string()).await.is_err() { break; }
                    }
                }
                _ = interval.tick() => {
                    if session.ping(b"").await.is_err() { break; }
                }
            }
        }
    });

    Ok(response)
}

pub async fn run_api_server(
    db: Arc<Database>,
    telemetry_broadcaster: broadcast::Sender<serde_json::Value>,
    mcu_cmd_sender: mpsc::Sender<HostToMcu>,
    machine_state: Arc<RwLock<MachineState>>,
) -> Result<()> {
    info!("Starting Moonraker-Compatible Actix-Web server on 0.0.0.0:7125");

    let app_state = web::Data::new(AppState {
        db,
        telemetry_broadcaster,
        mcu_cmd_sender,
        machine_state,
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin()
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(web::resource("/websocket").to(websocket_route))
            .route("/printer/info", web::get().to(get_printer_info))
            .route("/printer/objects/query", web::get().to(query_printer_objects))
            .route("/printer/gcode/script", web::post().to(post_gcode_script))
            .route("/server/info", web::get().to(get_server_info))
            .route("/server/files/list", web::get().to(get_server_files_list))
    })
    .bind("0.0.0.0:7125")?
    .run()
    .await?;

    Ok(())
}
