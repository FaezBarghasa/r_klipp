use anyhow::Result;
use log::{error, info};
use std::sync::Arc;
use std::thread;
use tokio::sync::{broadcast, mpsc, RwLock};

mod api;
mod bridge;
pub mod components;
mod db;
pub mod openpnp;

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting r_klipp host-server...");

    // Create a new Tokio runtime for background tasks (SerialBridge, channels)
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()?;

    // Shared state for all components
    let (telemetry_tx, _telemetry_rx) = broadcast::channel(1024);
    let (mcu_cmd_tx, mcu_cmd_rx) = mpsc::channel(1024);
    let machine_state = Arc::new(RwLock::new(api::MachineState::default()));

    // Clone handles for Actix server thread
    let api_telemetry_tx = telemetry_tx.clone();
    let api_mcu_cmd_tx = mcu_cmd_tx.clone();
    let api_machine_state = machine_state.clone();

    // 1. Initialize SurrealDB on Tokio runtime
    let db_path = "./data/r_klipp.db";
    let db = rt.block_on(async {
        let database = db::Database::new(db_path).await.expect("Failed to initialize SurrealDB");
        database.init_schema().await.expect("Failed to initialize SurrealDB schema");
        Arc::new(database)
    });
    info!("SurrealDB initialized at {}", db_path);

    let api_db = db.clone();

    // Initialize Moonraker components
    let file_manager = Arc::new(components::FileManager::new("./data/gcodes", "./data/config"));
    let job_queue = Arc::new(components::JobQueue::new());
    let data_store = Arc::new(components::DataStore::new(1200.0)); // 20-min temperature history

    let api_file_mgr = file_manager.clone();
    let api_job_queue = job_queue.clone();
    let api_data_store = data_store.clone();

    // 2. Initialize and spawn SerialBridge on Tokio
    let serial_port_path = "/dev/ttyUSB0".to_string();
    let baud_rate = 115200;
    let serial_bridge = bridge::SerialBridge::new(
        serial_port_path.clone(),
        baud_rate,
        telemetry_tx.clone(),
        mcu_cmd_rx,
        machine_state.clone(),
    );
    rt.spawn(async move {
        if let Err(e) = serial_bridge.run().await {
            error!("SerialBridge task ended: {:?}", e);
        }
    });

    // 3. Spawn Actix-Web server on dedicated system thread
    thread::spawn(move || {
        let sys = actix_web::rt::System::new();
        if let Err(e) = sys.block_on(api::run_api_server(
            api_db,
            api_telemetry_tx,
            api_mcu_cmd_tx,
            api_machine_state,
            api_file_mgr,
            api_job_queue,
            api_data_store,
        )) {
            error!("Actix-Web server error: {:?}", e);
        }
    });

    // 4. Run Slint UI on the main thread
    info!("Starting Slint UI on main thread...");
    let (ui_cmd_tx, mut ui_cmd_rx) = mpsc::channel::<host_ui::HostToMcu>(1024);
    let mcu_tx = mcu_cmd_tx.clone();
    rt.spawn(async move {
        while let Some(host_ui::HostToMcu::GCode(cmd)) = ui_cmd_rx.recv().await {
            let _ = mcu_tx.send(bridge::HostToMcu::GCode(cmd)).await;
        }
    });

    if let Err(e) = rt.block_on(host_ui::run_ui(ui_cmd_tx)) {
        error!("Slint UI failed: {:?}", e);
    }

    info!("r_klipp host-server shut down cleanly.");
    Ok(())
}
