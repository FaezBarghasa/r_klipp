use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use tokio::sync::{broadcast, mpsc, RwLock};

mod api;
mod bridge;
mod db;

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting r_klipp host-server...");

    // Create a new Tokio runtime for background tasks (Actix-Web, SerialBridge)
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // Shared state for all components
    let (telemetry_tx, _telemetry_rx) = broadcast::channel(1024); // For MCU -> API/UI telemetry
    let (mcu_cmd_tx, mcu_cmd_rx) = mpsc::channel(1024); // For API/UI -> MCU commands
    let machine_state = Arc::new(RwLock::new(api::MachineState::default()));

    // Clone senders for background tasks
    let api_telemetry_tx = telemetry_tx.clone();
    let api_mcu_cmd_tx = mcu_cmd_tx.clone();
    let api_machine_state = machine_state.clone();

    let bridge_telemetry_tx = telemetry_tx.clone();
    let bridge_mcu_cmd_rx = mcu_cmd_rx; // Only one receiver for mpsc
    let bridge_machine_state = machine_state.clone();

    // Spawn background tasks onto the Tokio runtime
    rt.spawn(async move {
        // 1. Initialize SurrealDB
        let db_path = "./data/r_klipp.db";
        let db = Arc::new(db::Database::new(db_path).await.expect("Failed to initialize SurrealDB"));
        info!("SurrealDB initialized at {}", db_path);

        // Run schema migrations
        db.init_schema().await.expect("Failed to initialize SurrealDB schema");
        info!("SurrealDB schema initialized.");

        // 2. Initialize and spawn SerialBridge
        let serial_port_path = "/dev/ttyUSB0".to_string(); // TODO: Make configurable
        let baud_rate = 115200; // TODO: Make configurable
        let serial_bridge = bridge::SerialBridge::new(
            serial_port_path.clone(),
            baud_rate,
            bridge_telemetry_tx,
            bridge_mcu_cmd_rx,
            bridge_machine_state,
        );
        tokio::spawn(async move {
            if let Err(e) = serial_bridge.run().await {
                error!("SerialBridge task failed: {:?}", e);
            }
        });
        info!("SerialBridge task spawned for {}", serial_port_path);

        // 3. Spawn Actix-Web server
        if let Err(e) = api::run_api_server(
            db,
            api_telemetry_tx,
            api_mcu_cmd_tx,
            api_machine_state,
        )
        .await
        {
            error!("Actix-Web server failed: {:?}", e);
        }
    });

    // Run Slint UI on the main thread
    info!("Starting Slint UI on main thread...");
    if let Err(e) = rt.block_on(host_ui::run_ui(mcu_cmd_tx)) {
        error!("Slint UI failed: {:?}", e);
    }

    info!("r_klipp host-server shut down cleanly.");

    Ok(())
}
