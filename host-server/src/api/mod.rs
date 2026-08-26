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
use crate::components::{
    DataStore, FileManager, JobQueue, MachineManager, PowerManager, SpoolmanClient, UpdateManager,
};

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
    pub file_manager: Arc<FileManager>,
    pub job_queue: Arc<JobQueue>,
    pub data_store: Arc<DataStore>,
    pub machine_mgr: Arc<MachineManager>,
    pub power_mgr: Arc<PowerManager>,
    pub update_mgr: Arc<UpdateManager>,
    pub spoolman: Arc<SpoolmanClient>,
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

pub async fn handle_jsonrpc_request(
    req: JsonRpcRequest,
    state: &AppState,
) -> serde_json::Value {
    let method = &req.method;
    let req_id = req.id.clone().unwrap_or(serde_json::Value::Null);

    let result = match method.as_str() {
        "server.info" => json!({
            "klippy_state": "ready",
            "klippy_connected": true,
            "api_version": [0, 1, 0],
            "api_version_string": "0.1.0-rklipp",
            "hostname": "r-klipp-host",
            "plugins": ["database", "file_manager", "gcode", "data_store", "job_queue"]
        }),
        "printer.info" => {
            let ms = state.machine_state.read().await;
            json!({
                "state": if ms.is_printing { "printing" } else { "ready" },
                "state_message": ms.state_message,
                "hostname": "r-klipp-host",
                "software_version": "0.1.0-rklipp"
            })
        }
        "printer.objects.query" => {
            let ms = state.machine_state.read().await;
            json!({
                "status": {
                    "toolhead": {
                        "position": ms.toolhead.position,
                        "homed_axes": ms.toolhead.homed_axes
                    },
                    "extruder": {
                        "temperature": ms.nozzle_temp,
                        "target": ms.nozzle_target
                    },
                    "heater_bed": {
                        "temperature": ms.bed_temp,
                        "target": ms.bed_target
                    }
                }
            })
        }
        "server.files.list" => {
            if let Ok(files) = state.file_manager.list_gcodes().await {
                json!(files)
            } else {
                json!([])
            }
        }
        "server.job_queue.status" => {
            let queue = state.job_queue.get_queue().await;
            let current = state.job_queue.get_current_job().await;
            json!({
                "queued_jobs": queue,
                "current_job": current
            })
        }
        "server.temperature_store" => {
            let history = state.data_store.get_all_sensors().await;
            json!(history)
        }
        "machine.proc_stats" => {
            let stats = state.machine_mgr.get_proc_stats().await;
            json!(stats)
        }
        "machine.system_info" => {
            let info = state.machine_mgr.get_system_info().await;
            json!(info)
        }
        "machine.device_power.devices" => {
            let devs = state.power_mgr.get_device_list().await;
            json!({ "devices": devs })
        }
        "machine.update.status" => {
            let status = state.update_mgr.get_status().await;
            json!(status)
        }
        "server.spoolman.get_spool" => {
            let spool = state.spoolman.get_active_spool().await;
            json!({ "spool": spool })
        }
        _ => {
            return json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                },
                "id": req_id
            });
        }
    };

    json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": req_id
    })
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
    let app_state_for_ws = state.clone();

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
                            if let Ok(rpc_req) = serde_json::from_str::<JsonRpcRequest>(&text) {
                                let rpc_resp = handle_jsonrpc_request(rpc_req, &app_state_for_ws).await;
                                if session.text(rpc_resp.to_string()).await.is_err() { break; }
                            } else {
                                info!("WebSocket raw client message: {}", text);
                            }
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

pub mod typed_query;
pub use typed_query::TypedQuery;

#[derive(Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: Option<String>,
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
}

/// POST /server/jsonrpc -> JSON-RPC 2.0 endpoint for HTTP clients
pub async fn post_jsonrpc(
    req: web::Json<JsonRpcRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let rpc_req = req.into_inner();
    let response = handle_jsonrpc_request(rpc_req, &state).await;
    HttpResponse::Ok().json(response)
}

pub async fn run_api_server(
    db: Arc<Database>,
    telemetry_broadcaster: broadcast::Sender<serde_json::Value>,
    mcu_cmd_sender: mpsc::Sender<HostToMcu>,
    machine_state: Arc<RwLock<MachineState>>,
    file_manager: Arc<FileManager>,
    job_queue: Arc<JobQueue>,
    data_store: Arc<DataStore>,
    machine_mgr: Arc<MachineManager>,
    power_mgr: Arc<PowerManager>,
    update_mgr: Arc<UpdateManager>,
    spoolman: Arc<SpoolmanClient>,
) -> Result<()> {
    info!("Starting Moonraker-Compatible Actix-Web server on 0.0.0.0:7125");

    let app_state = web::Data::new(AppState {
        db,
        telemetry_broadcaster,
        mcu_cmd_sender,
        machine_state,
        file_manager,
        job_queue,
        data_store,
        machine_mgr,
        power_mgr,
        update_mgr,
        spoolman,
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
            .route("/server/jsonrpc", web::post().to(post_jsonrpc))
    })
    .bind("0.0.0.0:7125")?
    .run()
    .await?;

    Ok(())
}
