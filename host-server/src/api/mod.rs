//! Moonraker-compatible HTTP & WebSocket API endpoints for Fluidd/Mainsail/KlipperScreen.

use actix_cors::Cors;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer};
use actix_ws::Message;
use anyhow::Result;
use futures_util::{StreamExt, TryStreamExt};
use log::info;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{sync::Arc, time::Duration};
use tokio::sync::{broadcast, mpsc, RwLock};

use crate::bridge::HostToMcu;
use crate::components::{
    DataStore, FileManager, JobQueue, MachineManager, PowerManager, SpoolmanClient, UpdateManager,
};
use crate::db::models::{GCodeFile, GCodeMetadata};
use crate::db::Database;

pub mod typed_query;


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
            "software_version": env!("CARGO_PKG_VERSION")
        }
    }))
}

/// GET /printer/objects/list -> List available printer objects
pub async fn list_printer_objects() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "result": {
            "objects": [
                "extruder",
                "heater_bed",
                "toolhead",
                "print_stats",
                "fan",
                "display_status",
                "virtual_sdcard",
                "webhooks",
                "configfile",
                "gcode_move",
                "idle_timeout",
                "motion_report"
            ]
        }
    }))
}

/// Query parameters for /printer/objects/query
#[derive(Deserialize)]
pub struct ObjectQueryParams {
    pub objects: Option<String>,
}

/// Build status JSON object
pub fn build_printer_status(ms: &MachineState) -> Value {
    json!({
        "webhooks": {
            "state": if ms.is_printing { "printing" } else { "ready" },
            "state_message": ms.state_message
        },
        "extruder": {
            "temperature": ms.nozzle_temp,
            "target": ms.nozzle_target,
            "power": 0.0,
            "can_extrude": ms.nozzle_temp > 170.0,
            "pressure_advance": 0.0,
            "smooth_time": 0.04
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
            "homed_axes": ms.toolhead.homed_axes,
            "print_time": 0.0,
            "estimated_print_time": 0.0
        },
        "print_stats": {
            "filename": ms.current_print_file,
            "progress": ms.print_progress,
            "state": if ms.is_printing { "printing" } else { "standby" },
            "message": "",
            "info": { "total_layer": null, "current_layer": null }
        },
        "display_status": {
            "progress": ms.print_progress,
            "message": ms.state_message
        },
        "virtual_sdcard": {
            "file_path": ms.current_print_file,
            "progress": ms.print_progress,
            "is_active": ms.is_printing,
            "file_position": 0,
            "file_size": 0
        },
        "gcode_move": {
            "speed": 0.0,
            "speed_factor": 1.0,
            "extrude_factor": 1.0,
            "absolute_coordinates": true,
            "absolute_extrude": true,
            "position": ms.toolhead.position,
            "gcode_position": ms.toolhead.position
        },
        "idle_timeout": {
            "state": "Ready",
            "printing_time": 0.0
        },
        "motion_report": {
            "live_position": ms.toolhead.position,
            "live_velocity": 0.0
        }
    })
}

/// GET /printer/objects/query -> Query hardware objects state
pub async fn query_printer_objects(
    state: web::Data<AppState>,
    query: web::Query<ObjectQueryParams>,
) -> HttpResponse {
    let ms = state.machine_state.read().await;
    let full_status = build_printer_status(&ms);

    let filtered_status = if let Some(ref obj_str) = query.objects {
        let mut map = serde_json::Map::new();
        for item in obj_str.split('&') {
            let key = item.split('=').next().unwrap_or("").trim();
            if !key.is_empty() {
                if let Some(val) = full_status.get(key) {
                    map.insert(key.to_string(), val.clone());
                }
            }
        }
        if map.is_empty() {
            full_status
        } else {
            Value::Object(map)
        }
    } else {
        full_status
    };

    HttpResponse::Ok().json(json!({
        "result": {
            "status": filtered_status
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
            let _ = state
                .mcu_cmd_sender
                .send(HostToMcu::GCode(line.to_string()))
                .await;
        }
    }

    HttpResponse::Ok().json(json!({ "result": "ok" }))
}

/// POST /printer/emergency_stop -> Emergency stop
pub async fn emergency_stop(state: web::Data<AppState>) -> HttpResponse {
    let _ = state
        .mcu_cmd_sender
        .send(HostToMcu::GCode("M112".to_string()))
        .await;
    let mut ms = state.machine_state.write().await;
    ms.is_printing = false;
    ms.state_message = "Shutdown due to emergency stop (M112)".to_string();
    HttpResponse::Ok().json(json!({ "result": "ok" }))
}

/// POST /printer/restart -> Host restart
pub async fn printer_restart() -> HttpResponse {
    HttpResponse::Ok().json(json!({ "result": "ok" }))
}

/// POST /printer/firmware_restart -> Firmware restart
pub async fn firmware_restart(state: web::Data<AppState>) -> HttpResponse {
    let _ = state
        .mcu_cmd_sender
        .send(HostToMcu::GCode("M112".to_string()))
        .await;
    HttpResponse::Ok().json(json!({ "result": "ok" }))
}

/// GET /server/info -> Moonraker Server Info
pub async fn get_server_info() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "result": {
            "klippy_state": "ready",
            "klippy_connected": true,
            "api_version": [0, 1, 0],
            "api_version_string": env!("CARGO_PKG_VERSION"),
            "hostname": "r-klipp-host",
            "plugins": [
                "database",
                "file_manager",
                "gcode",
                "data_store",
                "job_queue",
                "power",
                "update_manager",
                "spoolman"
            ]
        }
    }))
}

/// GET /server/files/list -> List stored G-Code files with metadata
pub async fn get_server_files_list(state: web::Data<AppState>) -> HttpResponse {
    match state.file_manager.list_gcodes().await {
        Ok(files) => HttpResponse::Ok().json(json!({ "result": files })),
        Err(e) => HttpResponse::InternalServerError().json(json!({ "error": e.to_string() })),
    }
}

/// GET /server/files/metadata -> Get G-Code metadata for specific file
#[derive(Deserialize)]
pub struct MetadataQuery {
    pub filename: String,
}

pub async fn get_file_metadata(
    state: web::Data<AppState>,
    query: web::Query<MetadataQuery>,
) -> HttpResponse {
    match state.file_manager.get_gcode_metadata(&query.filename).await {
        Ok(meta) => HttpResponse::Ok().json(json!({ "result": meta })),
        Err(e) => HttpResponse::NotFound().json(json!({ "error": e.to_string() })),
    }
}

/// GET /server/files/roots -> List available root sandboxes
pub async fn get_files_roots() -> HttpResponse {
    HttpResponse::Ok().json(json!({
        "result": [
            { "name": "gcodes", "permissions": "rw" },
            { "name": "config", "permissions": "rw" }
        ]
    }))
}

/// POST /server/files/upload -> Multipart G-Code file upload
pub async fn upload_file(
    mut payload: Multipart,
    state: web::Data<AppState>,
) -> HttpResponse {
    let mut filename = String::new();
    let mut file_data = Vec::new();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_disposition = field.content_disposition();
        if let Some(name) = content_disposition.and_then(|cd| cd.get_filename()) {
            filename = name.to_string();
        }

        while let Some(chunk) = field.next().await {
            if let Ok(data) = chunk {
                file_data.extend_from_slice(&data);
            }
        }
    }

    if filename.is_empty() {
        filename = format!("upload_{}.gcode", uuid::Uuid::new_v4());
    }

    if let Err(e) = state.file_manager.write_file("gcodes", &filename, &file_data).await {
        return HttpResponse::InternalServerError().json(json!({ "error": e.to_string() }));
    }

    if let Ok(meta) = state.file_manager.get_gcode_metadata(&filename).await {
        let _ = state.db.save_gcode_metadata(GCodeFile {
            id: None,
            path: filename.clone(),
            size: file_data.len() as u64,
            upload_date: chrono::Utc::now(),
            metadata: GCodeMetadata {
                estimated_time: meta.estimated_time.map(|t| t as u32),
                layer_height: meta.layer_height.map(|l| l as f32),
                filament_length: meta.filament_total.map(|f| f as f32),
                thumbnails: vec![],
            },
        }).await;

        HttpResponse::Ok().json(json!({ "result": { "item": meta } }))
    } else {
        HttpResponse::Ok().json(json!({ "result": "ok" }))
    }
}

/// DELETE /server/files/{root}/{filename:.*} -> Delete file
pub async fn delete_file(
    path: web::Path<(String, String)>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let (root, filename) = path.into_inner();
    match state.file_manager.delete_file(&root, &filename).await {
        Ok(_) => HttpResponse::Ok().json(json!({ "result": "ok" })),
        Err(e) => HttpResponse::BadRequest().json(json!({ "error": e.to_string() })),
    }
}

// ------------------------------------------------------------------------------------------------
// JSON-RPC 2.0 Dispatcher
// ------------------------------------------------------------------------------------------------

#[derive(Deserialize, Serialize, Debug)]
pub struct JsonRpcRequest {
    pub jsonrpc: Option<String>,
    pub method: String,
    pub params: Option<serde_json::Value>,
    pub id: Option<serde_json::Value>,
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
            "api_version_string": env!("CARGO_PKG_VERSION"),
            "hostname": "r-klipp-host",
            "plugins": [
                "database",
                "file_manager",
                "gcode",
                "data_store",
                "job_queue",
                "power",
                "update_manager",
                "spoolman"
            ]
        }),
        "printer.info" => {
            let ms = state.machine_state.read().await;
            json!({
                "state": if ms.is_printing { "printing" } else { "ready" },
                "state_message": ms.state_message,
                "hostname": "r-klipp-host",
                "software_version": env!("CARGO_PKG_VERSION")
            })
        }
        "printer.objects.list" => json!({
            "objects": [
                "extruder",
                "heater_bed",
                "toolhead",
                "print_stats",
                "fan",
                "display_status",
                "virtual_sdcard",
                "webhooks",
                "configfile",
                "gcode_move",
                "idle_timeout",
                "motion_report"
            ]
        }),
        "printer.objects.query" => {
            let ms = state.machine_state.read().await;
            let full_status = build_printer_status(&ms);
            json!({ "status": full_status })
        }
        "printer.gcode.script" => {
            if let Some(params) = &req.params {
                if let Some(script) = params.get("script").and_then(|s| s.as_str()) {
                    for line in script.lines() {
                        let line = line.trim();
                        if !line.is_empty() && !line.starts_with(';') {
                            let _ = state
                                .mcu_cmd_sender
                                .send(HostToMcu::GCode(line.to_string()))
                                .await;
                        }
                    }
                }
            }
            json!("ok")
        }
        "printer.emergency_stop" => {
            let _ = state
                .mcu_cmd_sender
                .send(HostToMcu::GCode("M112".to_string()))
                .await;
            let mut ms = state.machine_state.write().await;
            ms.is_printing = false;
            ms.state_message = "Shutdown due to emergency stop (M112)".to_string();
            json!("ok")
        }
        "printer.restart" | "printer.firmware_restart" => {
            json!("ok")
        }
        "server.files.list" => {
            if let Ok(files) = state.file_manager.list_gcodes().await {
                json!(files)
            } else {
                json!([])
            }
        }
        "server.files.roots" => json!([
            { "name": "gcodes", "permissions": "rw" },
            { "name": "config", "permissions": "rw" }
        ]),
        "server.files.metadata" => {
            let filename = req
                .params
                .as_ref()
                .and_then(|p| p.get("filename"))
                .and_then(|f| f.as_str())
                .unwrap_or("");
            if let Ok(meta) = state.file_manager.get_gcode_metadata(filename).await {
                json!(meta)
            } else {
                json!({})
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
        "server.job_queue.enqueue" => {
            let filename = req
                .params
                .as_ref()
                .and_then(|p| p.get("filename"))
                .and_then(|f| f.as_str())
                .unwrap_or("unknown.gcode");
            let job = state.job_queue.enqueue(filename).await;
            json!(job)
        }
        "server.job_queue.start" => {
            let job = state.job_queue.start_next_job().await;
            json!(job)
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
        "machine.shutdown" => {
            let _ = state.machine_mgr.system_shutdown().await;
            json!("ok")
        }
        "machine.reboot" => {
            let _ = state.machine_mgr.system_reboot().await;
            json!("ok")
        }
        "machine.device_power.devices" => {
            let devs = state.power_mgr.get_device_list().await;
            json!({ "devices": devs })
        }
        "machine.device_power.status" => {
            let devs = state.power_mgr.get_device_list().await;
            let mut map = serde_json::Map::new();
            for dev in devs {
                map.insert(dev.device, Value::String(dev.status));
            }
            Value::Object(map)
        }
        "machine.device_power.on" => {
            let dev_name = req
                .params
                .as_ref()
                .and_then(|p| p.get("device"))
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let ms = state.machine_state.read().await;
            match state.power_mgr.set_device_state(dev_name, "on", ms.is_printing).await {
                Ok(dev) => json!({ dev.device: dev.status }),
                Err(e) => {
                    return json!({
                        "jsonrpc": "2.0",
                        "error": { "code": -32000, "message": e.to_string() },
                        "id": req_id
                    });
                }
            }
        }
        "machine.device_power.off" => {
            let dev_name = req
                .params
                .as_ref()
                .and_then(|p| p.get("device"))
                .and_then(|d| d.as_str())
                .unwrap_or("");
            let ms = state.machine_state.read().await;
            match state.power_mgr.set_device_state(dev_name, "off", ms.is_printing).await {
                Ok(dev) => json!({ dev.device: dev.status }),
                Err(e) => {
                    return json!({
                        "jsonrpc": "2.0",
                        "error": { "code": -32000, "message": e.to_string() },
                        "id": req_id
                    });
                }
            }
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

/// POST /server/jsonrpc -> JSON-RPC 2.0 endpoint for HTTP clients
pub async fn post_jsonrpc(
    req: web::Json<JsonRpcRequest>,
    state: web::Data<AppState>,
) -> HttpResponse {
    let rpc_req = req.into_inner();
    let response = handle_jsonrpc_request(rpc_req, &state).await;
    HttpResponse::Ok().json(response)
}

// ------------------------------------------------------------------------------------------------
// WebSocket Stream
// ------------------------------------------------------------------------------------------------

pub async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, actix_web::Error> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;
    let mut rx = state.telemetry_broadcaster.subscribe();
    let app_state_for_ws = state.clone();

    actix_web::rt::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(5));
        let mut proc_interval = tokio::time::interval(Duration::from_secs(2));

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
                        let notify = json!({
                            "jsonrpc": "2.0",
                            "method": "notify_status_update",
                            "params": [data]
                        });
                        if session.text(notify.to_string()).await.is_err() { break; }
                    }
                }
                _ = proc_interval.tick() => {
                    let stats = app_state_for_ws.machine_mgr.get_proc_stats().await;
                    let notify = json!({
                        "jsonrpc": "2.0",
                        "method": "notify_proc_stat",
                        "params": [stats]
                    });
                    if session.text(notify.to_string()).await.is_err() { break; }
                }
                _ = interval.tick() => {
                    if session.ping(b"").await.is_err() { break; }
                }
            }
        }
    });

    Ok(response)
}

// ------------------------------------------------------------------------------------------------
// Server Startup
// ------------------------------------------------------------------------------------------------

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
            // Printer endpoints
            .route("/printer/info", web::get().to(get_printer_info))
            .route("/printer/objects/list", web::get().to(list_printer_objects))
            .route("/printer/objects/query", web::get().to(query_printer_objects))
            .route("/printer/gcode/script", web::post().to(post_gcode_script))
            .route("/printer/emergency_stop", web::post().to(emergency_stop))
            .route("/printer/restart", web::post().to(printer_restart))
            .route("/printer/firmware_restart", web::post().to(firmware_restart))
            // Server endpoints
            .route("/server/info", web::get().to(get_server_info))
            .route("/server/files/list", web::get().to(get_server_files_list))
            .route("/server/files/roots", web::get().to(get_files_roots))
            .route("/server/files/metadata", web::get().to(get_file_metadata))
            .route("/server/files/upload", web::post().to(upload_file))
            .route("/server/files/{root}/{filename:.*}", web::delete().to(delete_file))
            .route("/server/jsonrpc", web::post().to(post_jsonrpc))
    })
    .bind("0.0.0.0:7125")?
    .run()
    .await?;

    Ok(())
}
