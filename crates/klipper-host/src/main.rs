//! # Klipper Host
//!
//! This is the main entry point for the Klipper host software. It is a Rust-based
//! application that performs the same role as the Python-based `klippy` in the
//! official Klipper project.
//!
//! ## Responsibilities
//!
//! The host software is responsible for:
//!
//! *   Parsing G-code files.
//! *   Handling motion planning and kinematics.
//! *   Managing the user interface (e.g., web interface).
//! *   Sending commands to the MCU.
//!
//! ## Concurrency
//!
//! The host software is highly concurrent, with different components running as
//! separate tasks. This allows it to handle multiple connections and process G-code
//! with low latency.

// Use mimalloc for better performance
#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use anyhow::Result;
use clap::{Parser, Subcommand};
use parking_lot::Mutex;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::info;

mod api;
mod batch;
mod config;
mod gcode;
mod kinematics;
mod mcu_client;
mod state;
mod virtual_printer;

use config::PrinterConfig;
use state::PrinterState;

/// A Rust-based host process for Klipper 3D printer firmware.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run the main Klipper host server process.
    Run(RunArgs),
    /// Process a G-code file in a batch and print the resulting MCU commands.
    Batch(batch::BatchArgs),
}

/// Arguments for the `run` command.
#[derive(Parser, Debug)]
struct RunArgs {
    /// Path to the Klipper printer configuration file.
    #[arg(short, long, default_value = "printer.cfg")]
    config_path: PathBuf,

    /// Run with a mock MCU for testing purposes.
    #[arg(long)]
    mock_mcu: bool,

    /// Web server port for the API and UI.
    #[arg(short, long, default_value_t = 7125)]
    web_port: u16,

    /// Path to the virtual printer Unix domain socket.
    #[arg(long, default_value = "/tmp/printer")]
    socket_path: String,
}

#[actix_rt::main]
async fn main() -> Result<()> {
    // Initialize the logging subscriber.
    tracing_subscriber::fmt::init();

    // Parse command-line arguments.
    let cli = Cli::parse();

    match cli.command {
        Commands::Run(args) => run_server(args).await,
        Commands::Batch(args) => batch::run_batch_processing(args).await,
    }
}

/// Initializes and runs all the main server components as concurrent tasks.
async fn run_server(args: RunArgs) -> Result<()> {
    info!("Starting Klipper Host server...");

    // --- Initialization ---

    // 1. Load configuration.
    info!("Loading configuration from: {:?}", args.config_path);
    let app_config = Arc::new(PrinterConfig::load(&args.config_path)?);

    // 2. Create shared printer state.
    let printer_state = Arc::new(Mutex::new(PrinterState::new()));

    // 3. Create communication channels.
    // Channel for G-code commands from various sources (API, socket) to the dispatcher.
    let (gcode_tx, gcode_rx) = mpsc::channel(100);
    // Channel for MCU commands from the dispatcher to the MCU client.
    let (mcu_tx, mcu_rx) = mpsc::channel(100);

    // --- Component Spawning ---

    // G-code Dispatcher: Receives G-code, processes it, and sends MCU commands.
    let mut gcode_dispatcher = gcode::GCodeDispatcher::new(app_config.clone(), printer_state.clone(), mcu_tx);
    tokio::spawn(async move {
        gcode_dispatcher.run(gcode_rx).await;
    });
    info!("G-code dispatcher started.");

    // MCU Client: Manages the connection to the MCU.
    if args.mock_mcu {
        info!("Using mock MCU.");
        tokio::spawn(mcu_client::run_mock_mcu(mcu_rx, printer_state.clone()));
    } else {
        info!("Connecting to real MCU...");
        let mcu_config = app_config.mcu.clone();
        tokio::spawn(mcu_client::run_mcu_client(mcu_config, mcu_rx, printer_state.clone()));
    }
    info!("MCU client started.");

    // Virtual Printer Socket: Provides a Unix socket for compatibility with other tools.
    let socket_gcode_tx = gcode_tx.clone();
    let socket_path = args.socket_path.clone();
    tokio::spawn(async move {
        if let Err(e) = virtual_printer::start_virtual_printer(&socket_path, socket_gcode_tx).await {
            tracing::error!("Virtual printer socket failed: {}", e);
        }
    });

    // API Server: Provides a REST and WebSocket API for web interfaces.
    let api_state = api::AppState {
        printer_state: printer_state.clone(),
        gcode_sender: gcode_tx.clone(),
    };
    let api_server = api::start_api_server(args.web_port, api_state)?;
    info!("API server started on port {}.", args.web_port);

    // --- Run and Shutdown ---

    // Wait for a shutdown signal (e.g., Ctrl+C).
    tokio::select! {
        _ = api_server => {
            info!("API server shut down.");
        }
        _ = tokio::signal::ctrl_c() => {
            info!("Shutdown signal received. Exiting.");
        }
    }

    // Cleanup the virtual printer socket file.
    let _ = std::fs::remove_file(args.socket_path);

    Ok(())
}