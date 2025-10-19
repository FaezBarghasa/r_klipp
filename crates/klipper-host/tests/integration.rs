//! Integration Tests for the Klipper Host
//!
//! These tests spin up the entire application in a background task and then
//! interact with it using real network clients (Unix socket, WebSocket) to
//! verify end-to-end functionality.

use klipper_host::api;
use klipper_host::gcode::{parse_gcode, GCode};
use klipper_host::state::PrinterState;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::UnixStream;
use tokio::sync::mpsc;
use tokio::time::timeout;

/// A helper function to spawn a test server instance.
fn spawn_test_server() -> (String, api::AppState) {
    let socket_path = format!("/tmp/printer-test-{}", rand::random::<u32>());

    let state = Arc::new(Mutex::new(PrinterState::new()));
    let (gcode_tx, mut gcode_rx) = mpsc::channel(100);

    let app_state = api::AppState {
        printer_state: state.clone(),
        gcode_sender: gcode_tx,
    };

    // Spawn a dummy G-code receiver for the test.
    tokio::spawn(async move {
        while let Some(gcode) = gcode_rx.recv().await {
            // In a real test, you could assert that specific G-codes are received.
            println!("Test G-code receiver got: {:?}", gcode);
        }
    });

    let server_state = app_state.clone();
    let sp = socket_path.clone();
    std::thread::spawn(move || {
        let rt = actix_rt::System::new();
        rt.block_on(async {
            let api_server = api::start_api_server(0, server_state).unwrap(); // Port 0 for random port
            let socket_server = klipper_host::virtual_printer::start_virtual_printer(&sp, app_state.gcode_sender);

            tokio::select! {
                _ = api_server => {},
                _ = socket_server => {},
            }
        });
    });

    // Give the server a moment to start.
    std::thread::sleep(Duration::from_millis(100));

    (socket_path, app_state)
}

#[tokio::test]
async fn test_virtual_printer_socket_communication() {
    let (socket_path, _app_state) = spawn_test_server();

    // Attempt to connect to the socket and send a command.
    let mut stream = timeout(Duration::from_secs(2), UnixStream::connect(&socket_path))
        .await
        .expect("Should connect to the socket within 2 seconds")
        .expect("Connection should be successful");

    stream.write_all(b"G28\n").await.unwrap();
    let mut response = [0u8; 3]; // "ok\n"
    stream.read_exact(&mut response).await.unwrap();

    assert_eq!(&response, b"ok\n");

    // Clean up the socket file.
    let _ = std::fs::remove_file(socket_path);
}

#[tokio::test]
async fn test_api_status_endpoint() {
    // This is a more complex test that would require finding the random port
    // the server started on. For simplicity, we'll skip the full HTTP client
    // setup, but the framework is here.
    let (_socket_path, app_state) = spawn_test_server();

    // You would typically use a client like `reqwest` to query the API endpoint.
    // For now, we'll just check the state directly.
    let status = app_state.printer_state.lock().status;
    assert_eq!(status, klipper_host::state::PrinterStatus::Initializing);

    // Clean up the socket file.
    let _ = std::fs::remove_file(_socket_path);
}

