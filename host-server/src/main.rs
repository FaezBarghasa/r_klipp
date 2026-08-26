use anyhow::Result;
use clap::Parser;
use log::{error, info};
use std::sync::Arc;
use std::thread;
use tokio::sync::{broadcast, mpsc, RwLock};

mod api;
mod bridge;
pub mod components;
pub mod config;
mod db;
pub mod ipc;
pub mod openpnp;

#[derive(Parser, Debug)]
#[command(author, version, about = "r_klipp Moonraker & OpenPnP Host Server")]
struct CliArgs {
    /// Serial port device path for MCU comms
    #[arg(short, long, default_value = "/dev/ttyUSB0")]
    serial: String,

    /// Baud rate for serial communication
    #[arg(short, long, default_value_t = 115200)]
    baud: u32,

    /// HTTP & WebSocket server bind address
    #[arg(long, default_value = "0.0.0.0")]
    host: String,

    /// HTTP & WebSocket server port
    #[arg(short, long, default_value_t = 7125)]
    port: u16,

    /// Data directory path for databases and uploads
    #[arg(long, default_value = "./data")]
    data_dir: String,

    /// Disable GUI / run headless (CLI-only)
    #[arg(long, default_value_t = false)]
    no_ui: bool,
}

fn main() -> Result<()> {
    env_logger::init();
    let args = CliArgs::parse();
    info!("Starting r_klipp host-server on {}:{}...", args.host, args.port);

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
    let gcodes_dir = format!("{}/gcodes", args.data_dir);
    let config_dir = format!("{}/config", args.data_dir);
    let db_path = format!("{}/r_klipp.db", args.data_dir);

    let db = rt.block_on(async {
        let database = db::Database::new(&db_path)
            .await
            .expect("Failed to initialize SurrealDB");
        database
            .init_schema()
            .await
            .expect("Failed to initialize SurrealDB schema");
        Arc::new(database)
    });
    info!("SurrealDB initialized at {}", db_path);

    let api_db = db.clone();

    // Initialize Moonraker components
    let file_manager = Arc::new(components::FileManager::new(gcodes_dir, config_dir));
    let job_queue = Arc::new(components::JobQueue::new());
    let data_store = Arc::new(components::DataStore::new(1200.0)); // 20-min temperature history
    let machine_mgr = Arc::new(components::MachineManager::new());
    let power_mgr = Arc::new(components::PowerManager::new());
    let update_mgr = Arc::new(components::UpdateManager::new());
    let spoolman = Arc::new(components::SpoolmanClient::new());

    let api_file_mgr = file_manager.clone();
    let api_job_queue = job_queue.clone();
    let api_data_store = data_store.clone();
    let api_machine_mgr = machine_mgr.clone();
    let api_power_mgr = power_mgr.clone();
    let api_update_mgr = update_mgr.clone();
    let api_spoolman = spoolman.clone();

    // 2. Initialize and spawn SerialBridge on Tokio
    let serial_bridge = bridge::SerialBridge::new(
        args.serial.clone(),
        args.baud,
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
            api_machine_mgr,
            api_power_mgr,
            api_update_mgr,
            api_spoolman,
        )) {
            error!("Actix-Web server error: {:?}", e);
        }
    });

    // 4. Run Slint UI on main thread or wait for Ctrl+C in headless mode
    if args.no_ui {
        info!("Running in headless mode (no UI). Waiting for SIGINT/SIGTERM...");
        rt.block_on(async {
            let _ = tokio::signal::ctrl_c().await;
            info!("Shutdown signal received.");
        });
    } else {
        info!("Starting Slint UI on main thread...");
        let (ui_cmd_tx, mut ui_cmd_rx) = mpsc::channel::<host_ui::HostToMcu>(1024);
        let mcu_tx = mcu_cmd_tx.clone();
        rt.spawn(async move {
            while let Some(host_ui::HostToMcu::GCode(cmd)) = ui_cmd_rx.recv().await {
                let _ = mcu_tx.send(bridge::HostToMcu::GCode(cmd)).await;
            }
        });

        if let Err(e) = rt.block_on(host_ui::run_ui(ui_cmd_tx)) {
            error!("Slint UI ended: {:?}", e);
        }
    }

    info!("r_klipp host-server shut down cleanly.");
    Ok(())
}
