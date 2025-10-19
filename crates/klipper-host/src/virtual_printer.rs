//! Virtual Printer Unix Domain Socket
//!
//! This module creates and manages the `/tmp/printer` socket, which emulates
//! a traditional serial port. This allows G-code senders like OctoPrint,
//! Pronterface, etc., to connect and send commands as if they were talking
//! directly to a printer.

use crate::gcode::{parse_gcode, GCode};
use anyhow::Result;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};
use tokio::sync::mpsc::Sender;
use tracing::{error, info};

/// Handles an individual client connection to the socket.
async fn handle_client(mut stream: UnixStream, gcode_sender: Sender<GCode>) {
    info!("Client connected to virtual printer socket.");
    let (reader, mut writer) = stream.split();
    let mut reader = BufReader::new(reader);
    let mut line_buf = String::new();

    loop {
        // Read lines from the client.
        match reader.read_line(&mut line_buf).await {
            Ok(0) => {
                info!("Client disconnected from socket.");
                break;
            }
            Ok(_) => {
                let line = line_buf.trim();
                if !line.is_empty() {
                    info!("Received from socket: {}", line);
                    // Parse the G-code and send it to the dispatcher.
                    if let Some(gcode) = parse_gcode(line) {
                        if let Err(e) = gcode_sender.send(gcode).await {
                            error!("Failed to send G-code from socket to dispatcher: {}", e);
                            break;
                        }
                    }
                    // Respond with "ok" to the client.
                    if let Err(e) = writer.write_all(b"ok\n").await {
                        error!("Failed to write 'ok' to socket: {}", e);
                        break;
                    }
                }
                line_buf.clear();
            }
            Err(e) => {
                error!("Failed to read from socket: {}", e);
                break;
            }
        }
    }
}

/// Starts the Unix domain socket listener.
pub async fn start_virtual_printer(path: &str, gcode_sender: Sender<GCode>) -> Result<()> {
    // Clean up any old socket file that might exist.
    if Path::new(path).exists() {
        let _ = std::fs::remove_file(path);
        info!("Removed existing socket file at {}", path);
    }

    let listener = UnixListener::bind(path)
        .map_err(|e| anyhow::anyhow!("Failed to bind to socket {}: {}", path, e))?;
    info!("Virtual printer socket listening on {}", path);

    // Accept new connections in a loop.
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let sender = gcode_sender.clone();
                tokio::spawn(handle_client(stream, sender));
            }
            Err(e) => {
                error!("Failed to accept client connection on socket: {}", e);
            }
        }
    }
}

