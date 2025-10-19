//! # API Server
//!
//! This module provides a web server that exposes a REST and WebSocket API for
//! interacting with the Klipper host software. It is compatible with popular web
//! interfaces like Fluidd and Mainsail.
//!
//! ## REST API
//!
//! The REST API provides the following endpoints:
//!
//! *   `GET /api/printer/status`: Get the current status of the printer.
//! *   `POST /api/gcode`: Send a G-code command to the printer.
//!
//! ## WebSocket API
//!
//! The WebSocket API provides a real-time stream of printer status updates. Clients
//! can connect to the `/ws` endpoint to receive these updates. The server also sends
//! periodic heartbeats to keep the connection alive.

use crate::gcode::{parse_gcode, GCode};
use crate::state::PrinterState;
use actix::prelude::*;
use actix_web::{get, post, web, App, Error, HttpRequest, HttpResponse, HttpServer, Responder};
use actix_web_actors::ws;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::Sender;
use tracing::{info, warn};

/// Interval for sending WebSocket heartbeats.
const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
/// Timeout for WebSocket clients. If no heartbeat is received, the client is disconnected.
const CLIENT_TIMEOUT: Duration = Duration::from_secs(10);

/// The application state shared across all API handlers.
#[derive(Clone)]
pub struct AppState {
    pub printer_state: Arc<Mutex<PrinterState>>,
    pub gcode_sender: Sender<GCode>,
}

/// The WebSocket actor for handling a single client connection.
struct WebSocketSession {
    /// The time of the last heartbeat received from the client.
    hb: Instant,
    /// The shared application state.
    app_state: AppState,
    /// The last printer state sent to the client, used to avoid sending duplicate updates.
    last_state: String,
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;

    /// Called when the actor is started.
    fn started(&mut self, ctx: &mut Self::Context) {
        self.hb(ctx);
        self.send_state_update(ctx);
    }
}

/// Handler for WebSocket messages.
impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => {
                self.hb = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_)) => {
                self.hb = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                info!("WS: Received text: {}", text);
                // Handle incoming messages here (e.g., for authentication).
            }
            Ok(ws::Message::Close(reason)) => {
                ctx.close(reason);
                ctx.stop();
            }
            _ => ctx.stop(),
        }
    }
}

impl WebSocketSession {
    /// Sends a heartbeat ping to the client every `HEARTBEAT_INTERVAL`.
    fn hb(&self, ctx: &mut ws::WebsocketContext<Self>) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |act, ctx| {
            if Instant::now().duration_since(act.hb) > CLIENT_TIMEOUT {
                warn!("WebSocket Client heartbeat failed, disconnecting!");
                ctx.stop();
                return;
            }
            ctx.ping(b"");
            act.send_state_update(ctx);
        });
    }

    /// Sends the current printer state to the client if it has changed.
    fn send_state_update(&mut self, ctx: &mut ws::WebsocketContext<Self>) {
        let state = self.app_state.printer_state.lock();
        if let Ok(json_state) = serde_json::to_string(&*state) {
            if json_state != self.last_state {
                ctx.text(json_state.clone());
                self.last_state = json_state;
            }
        }
    }
}

/// WebSocket endpoint handler.
async fn websocket_handler(
    req: HttpRequest,
    stream: web::Payload,
    data: web::Data<AppState>,
) -> Result<HttpResponse, Error> {
    ws::start(
        WebSocketSession {
            hb: Instant::now(),
            app_state: data.get_ref().clone(),
            last_state: String::new(),
        },
        &req,
        stream,
    )
}

/// REST endpoint to get the current printer status.
#[get("/api/printer/status")]
async fn printer_status(data: web::Data<AppState>) -> impl Responder {
    let state = data.printer_state.lock();
    HttpResponse::Ok().json(&*state)
}

/// REST endpoint to send a G-code command.
#[post("/api/gcode")]
async fn send_gcode(body: String, data: web::Data<AppState>) -> impl Responder {
    info!("Received G-code via API: {}", body);
    let lines = body.lines();
    for line in lines {
        if let Some(gcode) = parse_gcode(line) {
            if let Err(e) = data.gcode_sender.send(gcode).await {
                tracing::error!("Failed to send G-code from API to dispatcher: {}", e);
                return HttpResponse::InternalServerError().body("Failed to queue G-code command");
            }
        }
    }
    HttpResponse::Ok().json(serde_json::json!({"status": "ok"}))
}

/// Starts the Actix web server.
pub fn start_api_server(port: u16, app_state: AppState) -> std::io::Result<actix_web::dev::Server> {
    let server = HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(app_state.clone()))
            .service(printer_status)
            .service(send_gcode)
            .route("/ws", web::get().to(websocket_handler))
    })
        .bind(("0.0.0.0", port))?
        .run();
    Ok(server)
}