//! Simulation Harness Utilities
//!
//! Provides utilities to run a host binary or in-process component against the
//! simulated MCU and capture communication traces for analysis and CI validation.

use crate::fake_mcu::{McuCommand, McuResponse};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::process::Stdio;
use std::time::SystemTime;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{info, instrument};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TraceDirection {
    HostToMcu,
    McuToHost,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum TraceContent {
    Command(McuCommand),
    Response(McuResponse),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TraceEntry {
    pub timestamp: SystemTime,
    pub direction: TraceDirection,
    pub content: TraceContent,
}

/// Represents the host side of the simulation, responsible for
/// communicating with the SimMcu and capturing traces.
pub struct SimHost {
    socket_path: String,
    trace: Vec<TraceEntry>,
}

impl SimHost {
    /// Creates a new `SimHost` that will connect to the given socket path.
    pub fn new(socket_path: &str) -> Self {
        SimHost {
            socket_path: socket_path.to_string(),
            trace: Vec::new(),
        }
    }

    /// Runs a host binary as an external process.
    #[instrument(skip(self))]
    pub async fn run_external_host(&mut self, host_binary_path: &str, gcode_file: &str) -> Result<()> {
        info!(binary = %host_binary_path, "Running external host process");

        let mut command = tokio::process::Command::new(host_binary_path);
        command
            .arg("--mcu")
            .arg(&self.socket_path)
            .arg("--gcode")
            .arg(gcode_file)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        // We can't interact with an external host in this simple model,
        // this is more of a placeholder for a more complex harness.
        // For this example, we'll focus on in-process simulation.
        let status = command.status().await?;
        if !status.success() {
            anyhow::bail!("Host process failed");
        }
        Ok(())
    }

    /// Runs a simulation where the host logic is executed in-process.
    #[instrument(skip(self, host_logic))]
    pub async fn run_in_process<F>(&mut self, host_logic: F) -> Result<()>
    where
        F: FnOnce(tokio::io::BufReader<tokio::net::UnixStream>, tokio::io::WriteHalf<tokio::net::UnixStream>) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<()>> + Send>>,
    {
        info!("Connecting to SimMcu for in-process simulation...");
        let stream = UnixStream::connect(&self.socket_path).await?;
        let (reader, writer) = tokio::io::split(stream);
        let reader = BufReader::new(reader);

        // In a real scenario, we would likely have a proxy here to intercept
        // and log messages for the trace. For simplicity, we'll pass the
        // streams to the host logic and assume it will provide the trace.
        // This example will simulate sending a few commands directly.

        host_logic(reader, writer).await?;

        Ok(())
    }

    /// Adds an entry to the trace log.
    pub fn record_trace(&mut self, direction: TraceDirection, content: TraceContent) {
        self.trace.push(TraceEntry {
            timestamp: SystemTime::now(),
            direction,
            content,
        });
    }

    /// Dumps the recorded trace to a file.
    pub fn dump_trace(&self, path: &Path) -> Result<()> {
        info!(path = %path.display(), "Dumping trace file");
        let mut file = File::create(path)?;
        let json = serde_json::to_string_pretty(&self.trace)?;
        file.write_all(json.as_bytes())?;
        Ok(())
    }
}
