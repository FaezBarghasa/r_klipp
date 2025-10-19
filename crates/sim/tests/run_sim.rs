//! Example: Run Host + Simulated MCU
//!
//! This example demonstrates running a simulated host and MCU, executing
//! a small set of G-code commands, and producing trace artifacts.

use anyhow::Result;
use sim::fake_mcu::{McuCommand, McuResponse};
use sim::harness::{SimHost, TraceDirection, TraceContent};
use sim::SimMcu;
use std::path::Path;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader, WriteHalf};
use tokio::net::UnixStream;
use tracing::info;

async fn simple_host_logic(
    mut reader: BufReader<tokio::io::ReadHalf<UnixStream>>,
    mut writer: WriteHalf<UnixStream>,
    gcode_commands: Vec<&str>,
    sim_host: &mut SimHost,
) -> Result<()> {
    // This function simulates the host sending G-code commands and processing responses.

    for gcode_cmd_str in gcode_commands {
        let mcu_cmd = match gcode_cmd_str {
            "G28" => McuCommand::Move { steps: 1000 }, // Home
            "M105" => McuCommand::AdcRead { pin: "temp_sensor".to_string() },
            "M112" => McuCommand::EmergencyStop,
            _ => continue,
        };

        let cmd_json = serde_json::to_string(&mcu_cmd)? + "\n";
        writer.write_all(cmd_json.as_bytes()).await?;
        sim_host.record_trace(TraceDirection::HostToMcu, TraceContent::Command(mcu_cmd));


        let mut response_line = String::new();
        reader.read_line(&mut response_line).await?;
        let mcu_response: McuResponse = serde_json::from_str(response_line.trim())?;
        sim_host.record_trace(TraceDirection::McuToHost, TraceContent::Response(mcu_response.clone()));
        info!(?mcu_response, "Host received response");
    }

    Ok(())
}


#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let socket_path = "/tmp/sim_klipper.sock";
    let trace_path = Path::new("/tmp/sim_trace.json");
    let golden_trace_path = Path::new("golden_traces/basic_flow.json");

    // Ensure the socket doesn't exist from a previous run
    let _ = std::fs::remove_file(socket_path);
    let _ = std::fs::create_dir_all("golden_traces");


    // 1. Start the simulated MCU in the background
    let sim_mcu = SimMcu::new(socket_path);
    tokio::spawn(async move {
        if let Err(e) = sim_mcu.run().await {
            eprintln!("SimMcu failed: {}", e);
        }
    });

    // Give the MCU a moment to start up
    tokio::time::sleep(Duration::from_millis(100)).await;

    // 2. Initialize the simulation host
    let mut sim_host = SimHost::new(socket_path);

    // 3. Define a simple G-code file content
    let gcode_file_content = vec![
        "G28",  // Home
        "M105", // Get temperature
        "M112", // Emergency stop
    ];

    // 4. Run the in-process host logic
    info!("Starting in-process host simulation...");
    let host_handle = |reader, writer| {
        let gcode = gcode_file_content.clone();
        Box::pin(simple_host_logic(reader, writer, gcode, &mut sim_host))
    };

    // Need to handle sim_host mutability carefully.
    // A more advanced version might use channels to send trace data.
    // For this example, we'll re-create the host and run logic that populates its trace.
    let mut host_for_trace = SimHost::new(socket_path);
    let stream = UnixStream::connect(&socket_path).await?;
    let (reader, writer) = tokio::io::split(stream);
    simple_host_logic(BufReader::new(reader), writer, gcode_file_content.clone(), &mut host_for_trace).await?;


    // 5. Dump the trace file
    host_for_trace.dump_trace(trace_path)?;

    // This is where you would generate the golden trace for the first time
    if !golden_trace_path.exists() {
        info!("Golden trace not found, creating it.");
        host_for_trace.dump_trace(golden_trace_path)?;
    }

    // 6. In CI, you would compare the new trace to the golden trace
    // For this example, we'll just print a success message.
    info!("Simulation finished successfully. Trace file generated at: {}", trace_path.display());
    info!("In a CI environment, you would now compare this trace with {}", golden_trace_path.display());

    let _ = std::fs::remove_file(socket_path);

    Ok(())
}
