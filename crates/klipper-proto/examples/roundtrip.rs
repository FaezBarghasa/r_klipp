//! Example demonstrating a tokio-based in-memory roundtrip of commands and responses.

use klipper_proto::commands::{Command, Message, Response};
use klipper_proto::io::KlipperFramed;
use tokio::io::DuplexStream;
use futures::{SinkExt, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an in-memory duplex stream to simulate a client-server connection.
    let (client, server) = tokio::io::duplex(1024);

    // Spawn a task to act as the MCU (server).
    let handle = tokio::spawn(async move {
        let mut framed = KlipperFramed::new(server);
        println!("[Server] MCU simulator started. Waiting for commands...");

        // Process incoming messages from the client.
        while let Some(result) = framed.next().await {
            match result {
                Ok(Message::Command(cmd)) => {
                    println!("[Server] Received command: {:?}", cmd);
                    let response = match cmd {
                        Command::GetConfig => Message::Response(Response::Config {
                            is_config_valid: true,
                            mcu_version: 0x010000,
                            mcu_name: "klipper-proto-mcu".into(),
                        }),
                        Command::GCode(gcode) => {
                            if gcode.starts_with("M112") {
                                Message::Response(Response::GCodeError("Emergency stop".into()))
                            } else {
                                Message::Response(Response::GCodeOk)
                            }
                        }
                        _ => Message::Response(Response::Log("Unknown command".into())),
                    };
                    println!("[Server] Sending response: {:?}", response);
                    if let Err(e) = framed.send(response).await {
                        eprintln!("[Server] Failed to send response: {:?}", e);
                    }
                }
                Ok(Message::Response(_)) => {
                    // Server shouldn't receive responses, this is an error in protocol.
                    eprintln!("[Server] Error: Received unexpected response from client.");
                }
                Err(e) => {
                    eprintln!("[Server] Decoding error: {:?}", e);
                    break;
                }
            }
        }
        println!("[Server] Connection closed.");
    });

    // --- Client side logic ---
    let mut framed_client = KlipperFramed::new(client);

    // Test 1: GetConfig
    println!("[Client] Sending GetConfig command...");
    framed_client.send(Message::Command(Command::GetConfig)).await?;
    if let Some(Ok(Message::Response(resp))) = framed_client.next().await {
        println!("[Client] Received response: {:?}", resp);
        assert!(matches!(resp, Response::Config { .. }));
    }

    // Test 2: GCode command
    println!("\n[Client] Sending GCode command 'G28'...");
    framed_client.send(Message::Command(Command::GCode("G28".into()))).await?;
    if let Some(Ok(Message::Response(resp))) = framed_client.next().await {
        println!("[Client] Received response: {:?}", resp);
        assert_eq!(resp, Response::GCodeOk);
    }

    // Cleanly shutdown
    drop(framed_client);
    handle.await?;

    Ok(())
}

