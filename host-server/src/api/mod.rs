use actix_web::{web, App, HttpServer, Responder, HttpResponse};
use actix_cors::Cors;
use actix_ws::{Message, ProtocolError};
use futures_util::{StreamExt, SinkExt};
use tokio::sync::{broadcast, mpsc, RwLock};
use std::{sync::Arc, time::Duration};
use anyhow::Result;
use serde_json::json;
use uuid::Uuid;
use tokio::fs;
use tokio_util::codec::{BytesCodec, FramedRead};
use mime::Mime;
use log::{info, error};

use crate::db::{Database, HostError};
use crate::db::models::{GCodeFile, GCodeMetadata, PrintHistory, PrintStatus};
use crate::bridge::HostToMcu; // Assuming this will be defined in bridge module

// Placeholder for machine state
#[derive(Debug, Default)]
pub struct MachineState {
    pub nozzle_temp: f32,
    pub bed_temp: f32,
    pub current_print_file: Option<String>,
    pub print_progress: f32,
    // Add other relevant machine state fields
}

pub struct AppState {
    pub db: Arc<Database>,
    pub telemetry_broadcaster: broadcast::Sender<serde_json::Value>,
    pub mcu_cmd_sender: mpsc::Sender<HostToMcu>,
    pub machine_state: Arc<RwLock<MachineState>>,
}

async fn websocket_route(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, ProtocolError> {
    let (response, mut session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    let mut rx = state.telemetry_broadcaster.subscribe();
    let mcu_cmd_sender = state.mcu_cmd_sender.clone();

    actix_web::rt::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(10)); // Ping interval

        loop {
            tokio::select! {
                msg = msg_stream.next() => {
                    match msg {
                        Some(Ok(Message::Ping(bytes))) => {
                            if session.pong(&bytes).await.is_err() {
                                break;
                            }
                        }
                        Some(Ok(Message::Text(text))) => {
                            info!("Received WebSocket message: {}", text);
                            // Handle incoming messages from client if needed
                            // e.g., JSON-RPC commands over WebSocket
                        }
                        Some(Ok(Message::Close(_))) => {
                            break;
                        }
                        _ => break, // Connection error or other message types
                    }
                }
                telemetry = rx.recv() => {
                    match telemetry {
                        Ok(data) => {
                            if session.text(data.to_string()).await.is_err() {
                                break;
                            }
                        }
                        Err(broadcast::error::RecvError::Lagged(_)) => {
                            error!("WebSocket client lagged, dropping messages.");
                            // Optionally send a warning to the client or close connection
                        }
                        Err(_) => break, // Broadcaster closed
                    }
                }
                _ = interval.tick() => {
                    if session.ping(b"").await.is_err() {
                        break;
                    }
                }
            }
        }
        info!("WebSocket session ended.");
    });

    Ok(response)
}

async fn get_printer_info(state: web::Data<AppState>) -> Result<HttpResponse, HostError> {
    let machine_state = state.machine_state.read().await;
    Ok(HttpResponse::Ok().json(json!({
        "result": {
            "heater_bed": { "temperature": machine_state.bed_temp },
            "extruder": { "temperature": machine_state.nozzle_temp },
            "print_stats": {
                "filename": machine_state.current_print_file,
                "progress": machine_state.print_progress,
                "state": "printing" // Placeholder
            },
            // Add more printer info from machine_state
        }
    })))
}

async fn get_server_info() -> Result<HttpResponse, HostError> {
    Ok(HttpResponse::Ok().json(json!({
        "result": {
            "klippy_state": "ready", // Placeholder
            "api_version": "0.1.0",
            "api_version_string": "r_klipp host-server 0.1.0",
            "hostname": "r-klipp-host",
        }
    })))
}

async fn handle_rpc(
    body: web::Json<serde_json::Value>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, HostError> {
    let method = body["method"].as_str().unwrap_or_default();
    let id = body["id"].clone();

    match method {
        "printer.info" => get_printer_info(state).await,
        "server.info" => get_server_info().await,
        // Add more RPC methods as needed
        _ => Ok(HttpResponse::BadRequest().json(json!({
            "id": id,
            "error": {
                "code": -32601,
                "message": format!("Method not found: {}", method)
            }
        }))),
    }
}

async fn get_files(state: web::Data<AppState>) -> Result<HttpResponse, HostError> {
    let files = state.db.get_gcode_files().await?;
    Ok(HttpResponse::Ok().json(json!({ "files": files })))
}

async fn upload_file(
    mut payload: web::Payload,
    state: web::Data<AppState>,
) -> Result<HttpResponse, HostError> {
    let upload_dir = "./uploads"; // Define your upload directory
    fs::create_dir_all(upload_dir).await.map_err(|e| HostError::Other(e.to_string()))?;

    let mut filename = String::new();
    let mut file_content = Vec::new();
    let mut boundary = String::new();

    // This is a simplified multipart parser. A real implementation would use a dedicated crate.
    while let Some(item) = payload.next().await {
        let mut field = item.map_err(|e| HostError::Other(e.to_string()))?;
        if boundary.is_empty() {
            // Attempt to extract boundary from Content-Type header if available
            // For simplicity, we'll assume a basic boundary for now or expect it in the first chunk
            // A proper solution would parse the Content-Type header from the request.
            // For now, let's just assume the first line is the boundary.
            let s = String::from_utf8_lossy(&field);
            if let Some(line) = s.lines().next() {
                if line.starts_with("--") {
                    boundary = line.to_string();
                }
            }
        }

        // Simplified logic to extract filename and content
        // In a real app, use `actix-multipart` or similar
        let s = String::from_utf8_lossy(&field);
        if s.contains("filename=\"") {
            if let Some(start) = s.find("filename=\"") {
                let rest = &s[start + "filename=\"".len()..];
                if let Some(end) = rest.find("\"") {
                    filename = rest[..end].to_string();
                }
            }
        }
        file_content.extend_from_slice(&field);
    }

    if filename.is_empty() {
        return Err(HostError::Other("No filename found in multipart data".to_string()));
    }

    let file_path = format!("{}/{}", upload_dir, filename);
    fs::write(&file_path, &file_content).await.map_err(|e| HostError::Other(e.to_string()))?;

    let metadata = GCodeMetadata::default(); // Placeholder for actual parsing
    let gcode_file = GCodeFile {
        id: None,
        path: file_path.clone(),
        size: file_content.len() as u64,
        upload_date: Utc::now(),
        metadata,
    };

    state.db.save_gcode_metadata(gcode_file).await?;

    Ok(HttpResponse::Ok().json(json!({ "message": "File uploaded successfully", "path": file_path })))
}

async fn start_print(
    path: web::Path<String>,
    state: web::Data<AppState>,
) -> Result<HttpResponse, HostError> {
    let file_path = path.into_inner();
    info!("Attempting to start print for file: {}", file_path);

    let file = fs::File::open(&file_path).await.map_err(|e| HostError::Other(format!("Failed to open file: {}", e)))?;
    let reader = FramedRead::new(file, BytesCodec::new());

    let mut lines = reader.map(|r| r.map(|b| String::from_utf8_lossy(&b).to_string()));

    // Simulate sending G-code lines to MCU
    let mut mcu_cmd_sender = state.mcu_cmd_sender.clone();
    let machine_state = state.machine_state.clone();

    tokio::spawn(async move {
        let mut line_num = 0;
        while let Some(line) = lines.next().await {
            match line {
                Ok(gcode_line) => {
                    info!("Sending G-code line {}: {}", line_num, gcode_line.trim());
                    // In a real scenario, you'd parse and send specific commands
                    if let Err(e) = mcu_cmd_sender.send(HostToMcu::GCode(gcode_line.trim().to_string())).await {
                        error!("Failed to send G-code to MCU: {}", e);
                        break;
                    }
                    line_num += 1;
                    // Update print progress (simplified)
                    let mut state_guard = machine_state.write().await;
                    state_guard.print_progress = (line_num as f32 / 1000.0).min(1.0); // Assuming 1000 lines for simplicity
                    tokio::time::sleep(Duration::from_millis(50)).await; // Simulate print speed
                }
                Err(e) => {
                    error!("Error reading G-code file: {}", e);
                    break;
                }
            }
        }
        info!("Finished streaming G-code for print.");
        let mut state_guard = machine_state.write().await;
        state_guard.print_progress = 1.0;
        state_guard.current_print_file = None;

        // Save print history
        let db = state.db.clone(); // Clone Arc for async block
        let history = PrintHistory {
            id: None,
            start_time: Utc::now(), // Should be actual start time
            end_time: Some(Utc::now()),
            status: PrintStatus::Completed,
            telemetry_summary: Default::default(),
        };
        if let Err(e) = db.save_print_history(history).await {
            error!("Failed to save print history: {:?}", e);
        }
    });

    let mut state_guard = state.machine_state.write().await;
    state_guard.current_print_file = Some(file_path.clone());
    state_guard.print_progress = 0.0;

    Ok(HttpResponse::Ok().json(json!({ "message": format!("Print started for {}", file_path) })))
}


pub async fn run_api_server(
    db: Arc<Database>,
    telemetry_broadcaster: broadcast::Sender<serde_json::Value>,
    mcu_cmd_sender: mpsc::Sender<HostToMcu>,
    machine_state: Arc<RwLock<MachineState>>,
) -> Result<()> {
    info!("Starting Actix-Web server on 0.0.0.0:7125");

    let app_state = web::Data::new(AppState {
        db,
        telemetry_broadcaster,
        mcu_cmd_sender,
        machine_state,
    });

    HttpServer::new(move || {
        let cors = Cors::default()
            .allow_any_origin() // For development, allow any origin
            .allow_any_method()
            .allow_any_header()
            .max_age(3600);

        App::new()
            .wrap(cors)
            .app_data(app_state.clone())
            .service(web::resource("/websocket").to(websocket_route))
            .route("/api/rpc", web::post().to(handle_rpc))
            .route("/api/files", web::get().to(get_files))
            .route("/api/files/upload", web::post().to(upload_file))
            .route("/api/print/start/{file_path}", web::post().to(start_print))
    })
    .bind("0.0.0.0:7125")?
    .run()
    .await?;

    Ok(())
}
