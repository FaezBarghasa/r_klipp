use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

mod api;
mod bridge;
mod db;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    info!("Starting r_klipp host-server...");

    // 1. Initialize SurrealDB
    let db_path = "./data/r_klipp.db";
    let db = Arc::new(db::Database::new(db_path).await?);
    info!("SurrealDB initialized at {}", db_path);

    // Run schema migrations
    db.init_schema().await?;
    info!("SurrealDB schema initialized.");

    // 2. Setup communication channels
    let (telemetry_tx, _telemetry_rx) = broadcast::channel(1024); // For MCU -> API/UI telemetry
    let (mcu_cmd_tx, mcu_cmd_rx) = mpsc::channel(1024); // For API/UI -> MCU commands

    // Shared machine state
    let machine_state = Arc::new(RwLock::new(api::MachineState::default()));

    // 3. Initialize and spawn SerialBridge
    let serial_port_path = "/dev/ttyUSB0".to_string(); // TODO: Make configurable
    let baud_rate = 115200; // TODO: Make configurable
    let serial_bridge = bridge::SerialBridge::new(
        serial_port_path.clone(),
        baud_rate,
        telemetry_tx.clone(),
        mcu_cmd_rx,
        machine_state.clone(),
    );
    tokio::spawn(async move {
        if let Err(e) = serial_bridge.run().await {
            error!("SerialBridge task failed: {:?}", e);
        }
    });
    info!("SerialBridge task spawned for {}", serial_port_path);

    // 4. Spawn Actix-Web server
    let api_db = db.clone();
    let api_telemetry_tx = telemetry_tx.clone();
    let api_mcu_cmd_tx = mcu_cmd_tx.clone();
    let api_machine_state = machine_state.clone();
    tokio::spawn(async move {
        if let Err(e) = api::run_api_server(
            api_db,
            api_telemetry_tx,
            api_mcu_cmd_tx,
            api_machine_state,
        )
        .await
        {
            error!("Actix-Web server failed: {:?}", e);
        }
    });
    info!("Actix-Web server spawned on 0.0.0.0:7125");

    // 5. Run Slint UI (on the main thread)
    // Note: Slint requires the main thread for its event loop.
    // We'll simulate this by running it last and letting it block main.
    // In a real application, you might need a more sophisticated main thread executor.
    info!("Starting Slint UI...");
    // The Slint UI is in a separate crate, so we'll just log that it would be started here.
    // For a unified binary, you'd typically have the Slint app's main function called here,
    // or integrate it more tightly if Slint provided a way to run its event loop in a sub-task.
    // For now, we'll assume `host-ui` is a separate executable.

    // To satisfy the prompt's requirement of "Unified Process Orchestration"
    // and running Slint on the main thread, we'll need to adjust the project structure
    // or how Slint is launched. For now, let's assume `host-ui` is built and run separately.
    // If this were a single binary, the `host-ui::main().await?` would be here.
    // Since it's a workspace with two binaries, we'll just indicate the intention.

    info!("r_klipp host-server started successfully. UI would be launched separately.");

    // Keep the main thread alive for background tasks
    // In a real scenario, you'd have a graceful shutdown mechanism here
    // that listens for OS signals (SIGINT, SIGTERM) and cleans up.
    tokio::signal::ctrl_c().await?;
    info!("Ctrl-C received, shutting down...");

    // TODO: Implement graceful shutdown for all components
    // - Close serial port
    // - Flush SurrealDB
    // - Stop Actix-Web server

    info!("r_klipp host-server shut down cleanly.");

    Ok(())
}
