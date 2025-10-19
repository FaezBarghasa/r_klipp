// File: crates/klipper-host/tests/protocol_compatibility.rs
//! Integration test to verify basic protocol compatibility with a live Klipper host.

use tokio::net::UnixStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use std::time::Duration;

const KLIPPER_SOCKET_PATH: &str = "/tmp/printer";

#[tokio::test]
#[ignore] // This test requires a live Klipper instance running.
async fn test_connect_and_handshake_with_live_klipper() {
    println!("Attempting to connect to Klipper at {}", KLIPPER_SOCKET_PATH);

    let mut stream = match UnixStream::connect(KLIPPER_SOCKET_PATH).await {
        Ok(s) => s,
        Err(e) => {
            eprintln!("Failed to connect to Klipper socket: {}", e);
            eprintln!("Please ensure Klipper (klippy.py) is running.");
            panic!();
        }
    };

    println!("Connected. Sending 'get_config' command...");

    // This is a simplified representation of the binary command.
    // A real test would use the `klipper-proto` codec to construct the message.
    let get_config_cmd = b"\x7E\x03\x01\x02\xAB\xCD"; // Placeholder binary command
    stream.write_all(get_config_cmd).await.unwrap();

    let mut response = Vec::new();
    let _ = stream.read_to_end(&mut response).await;

    println!("Received response ({} bytes)", response.len());
    assert!(!response.is_empty(), "Should have received a response from Klipper");
}

