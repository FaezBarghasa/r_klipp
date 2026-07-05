use actix_web::{web, HttpResponse, Responder, HttpRequest};
use actix_cors::Cors;
use actix_ws::{Message, ProtocolError};
use anyhow::Result;
use log::{info, error};
use serde_json::json;
use tokio::sync::{broadcast, mpsc};
use tokio::time::{interval, Duration};
use futures_util::{StreamExt, SinkExt};
use std::sync::Arc;
use crate::db::{Database, HostError, GCodeFile};

// --- Telemetry Broadcaster ---
pub struct TelemetryBroadcaster {
    sender: broadcast::Sender<String>,
}

impl TelemetryBroadcaster {
    pub fn new() -> Self {
        let (sender, _receiver) = broadcast::channel(1024); // Buffer for 1024 messages
        TelemetryBroadcaster { sender }
    }

    pub fn send(&self, message: String) {
        if let Err(e) = self.sender.send(message) {
            error!("Failed to broadcast telemetry: {}", e);
        }
    }

    pub fn subscribe(&self) -> broadcast::Receiver<String> {
        self.sender.subscribe()
    }
}

// --- WebSocket Handler ---
async fn websocket_route(
    req: HttpRequest,
    stream: web::Payload,
    broadcaster: web::Data<Arc<TelemetryBroadcaster>>,
) -> Result<HttpResponse, ProtocolError> {
    let (res, session, mut msg_stream) = actix_ws::handle(&req, stream)?;

    // Spawn a task to send messages from the broadcaster to the client
    let mut rx = broadcaster.subscribe();
    tokio::spawn(async move {
        while let Ok(msg) = rx.recv().await {
            if session.text(msg).await.is_err() {
                break;
            }
        }
        let _ = session.close(None).await;
    });

    // Spawn a task to receive messages from the client (e.g., pings, control commands)
    tokio::spawn(async move {
        while let Some(Ok(msg)) = msg_stream.next().await {
            match msg {
                Message::Ping(bytes) => {
                    if session.pong(&bytes).await.is_err() {
                        break;
                    }
                }
                Message::Text(text) => {
                    info!("Received text from WebSocket: {}", text);
                    // Handle incoming commands from UI/web client if necessary
                }
                Message::Binary(bytes) => {
                    info!("Received binary from WebSocket: {:?}", bytes);
                }
                Message::Close(reason) => {
                    info!("WebSocket closed: {:?}", reason);
                    break;
                }
                _ => {}
            }
        }
        let _ = session.close(None).await;
    });

    Ok(res)
}

// --- API Handlers ---
async fn get_files(db: web::Data<Database>) -> Result<HttpResponse, HostError> {
    let files = db.get_gcode_files().await?;
    Ok(HttpResponse::Ok().json(files))
}

async fn upload_file(
    mut payload: web::Payload,
    db: web::Data<Database>,
) -> Result<HttpResponse, HostError> {
    // This is a placeholder. Actual multipart upload parsing is complex.
    // For now, we'll simulate saving a file and its metadata.
    info!("Received file upload request (payload not processed yet)");

    let dummy_file = GCodeFile {
        id: None,
        path: "/path/to/uploaded/dummy.gcode".into(),
        name: "dummy.gcode".into(),
        size: 12345,
        upload_date: chrono::Utc::now(),
        estimated_time_secs: Some(1200),
        thumbnail_path: None,
    };

    let saved_file = db.save_gcode_metadata(dummy_file).await?;

    Ok(HttpResponse::Ok().json(saved_file))
}

async fn rpc_handler(
    body: web::Json<serde_json::Value>,
    db: web::Data<Database>,
    // In a real scenario, you'd also pass the SerialBridge sender here
) -> Result<HttpResponse, HostError> {
    let method = body["method"].as_str().unwrap_or_default();
    info!("Received RPC method: {}", method);

    match method {
        "printer.info" => {
            Ok(HttpResponse::Ok().json(json!({
                "jsonrpc": "2.0",
                "result": {
                    "state": "ready",
                    "status": "operational",
                    "printer_name": "r_klipp_printer"
                },
                "id": body["id"]
            })))
        }
        "server.info" => {
            Ok(HttpResponse::Ok().json(json!({
                "jsonrpc": "2.0",
                "result": {
                    "klippy_state": "ready",
                    "moonraker_version": "r_klipp_v0.1.0",
                    "api_version": 1
                },
                "id": body["id"]
            })))
        }
        "printer.gcode.script" => {
            let script = body["params"]["script"].as_str().unwrap_or_default();
            info!("Executing G-code script: {}", script);
            // Here, you would send the script to the SerialBridge
            Ok(HttpResponse::Ok().json(json!({
                "jsonrpc": "2.0",
                "result": "ok",
                "id": body["id"]
            })))
        }
        _ => {
            Ok(HttpResponse::BadRequest().json(json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                },
                "id": body["id"]
            })))
        }
    }
}

// --- Service Configuration ---
pub fn api_service(broadcaster: Arc<TelemetryBroadcaster>) -> actix_web::Scope {
    let cors = Cors::default()
        .allow_any_origin()
        .allow_any_method()
        .allow_any_header()
        .max_age(3600);

    web::scope("/api")
        .wrap(cors)
        .app_data(web::Data::new(broadcaster.clone()))
        .route("/files", web::get().to(get_files))
        .route("/files/upload", web::post().to(upload_file))
        .route("/rpc", web::post().to(rpc_handler))
}

pub fn websocket_service(broadcaster: Arc<TelemetryBroadcaster>) -> actix_web::Scope {
    web::scope("/") // WebSocket is often at the root or a specific path
        .app_data(web::Data::new(broadcaster.clone()))
        .route("/websocket", web::get().to(websocket_route))
}

#[cfg(test)]
mod tests {
    use super::*;
    use actix_web::{test, web::Bytes};
    use crate::db::Database;
    use tokio::sync::broadcast;

    async fn setup_test_app(db: Database, broadcaster: Arc<TelemetryBroadcaster>) -> actix_web::App<impl actix_web::dev::ServiceFactory> {
        actix_web::App::new()
            .app_data(web::Data::new(db.clone()))
            .service(api_service(broadcaster.clone()))
            .service(websocket_service(broadcaster.clone()))
    }

    #[actix_web::test]
    async fn test_get_files_empty() {
        let db = Database::new().await.unwrap();
        db.init_schema().await.unwrap();
        let broadcaster = Arc::new(TelemetryBroadcaster::new());
        let app = test::init_service(setup_test_app(db, broadcaster)).await;

        let req = test::TestRequest::get().uri("/api/files").to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_ok());
        let response_body: Vec<GCodeFile> = test::read_body_json(resp).await;
        assert!(response_body.is_empty());
    }

    #[actix_web::test]
    async fn test_rpc_printer_info() {
        let db = Database::new().await.unwrap();
        db.init_schema().await.unwrap();
        let broadcaster = Arc::new(TelemetryBroadcaster::new());
        let app = test::init_service(setup_test_app(db, broadcaster)).await;

        let req = test::TestRequest::post()
            .uri("/api/rpc")
            .set_json(&json!({"jsonrpc": "2.0", "method": "printer.info", "id": 1}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_ok());
        let response_body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(response_body["result"]["state"], "ready");
    }

    #[actix_web::test]
    async fn test_rpc_gcode_script() {
        let db = Database::new().await.unwrap();
        db.init_schema().await.unwrap();
        let broadcaster = Arc::new(TelemetryBroadcaster::new());
        let app = test::init_service(setup_test_app(db, broadcaster)).await;

        let req = test::TestRequest::post()
            .uri("/api/rpc")
            .set_json(&json!({"jsonrpc": "2.0", "method": "printer.gcode.script", "params": {"script": "G28"}, "id": 1}))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert!(resp.status().is_ok());
        let response_body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(response_body["result"], "ok");
    }

    #[actix_web::test]
    async fn test_websocket_connection() {
        let db = Database::new().await.unwrap();
        db.init_schema().await.unwrap();
        let broadcaster = Arc::new(TelemetryBroadcaster::new());
        let app = test::init_service(setup_test_app(db, broadcaster.clone())).await;

        let mut ws = test::TestRequest::get().uri("/websocket").send_request(&app).await;
        assert!(ws.status().is_switching_protocols());

        // Test sending a message from broadcaster and receiving it on the client
        let test_message = "{\"nozzle_temp\": 200.0}".to_string();
        broadcaster.send(test_message.clone());

        let response = ws.next().await.unwrap().unwrap();
        assert_eq!(response, Message::Text(test_message));
    }
}