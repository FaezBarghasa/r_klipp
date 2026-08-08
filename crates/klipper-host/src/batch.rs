//! Batch G-code Processor
//!
//! A CLI subcommand for translating a G-code file into low-level Klipper
//! commands for testing and analysis, without needing a live printer connection.

use crate::config::PrinterConfig;
use crate::gcode::parse_gcode;
use crate::kinematics::{CartesianKinematics, Kinematics};
use crate::state::Position;
use anyhow::{Context, Result};
use clap::Parser;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::PathBuf;
use tracing::info;

/// Arguments for the `batch` subcommand.
#[derive(Parser, Debug)]
pub struct BatchArgs {
    /// Path to the G-code file to process.
    #[arg(required = true)]
    gcode_file: PathBuf,

    /// Path to the Klipper printer configuration file.
    #[arg(short, long, default_value = "printer.cfg")]
    config_path: PathBuf,
}

/// Runs the batch processing logic.
pub async fn run_batch_processing(args: BatchArgs) -> Result<()> {
    info!(
        "Starting batch processing for file: {:?}",
        args.gcode_file
    );
    info!("Using config file: {:?}", args.config_path);

    // Load printer configuration.
    let config = PrinterConfig::load(&args.config_path)?;

    // Initialize kinematics based on config.
    let kinematics = CartesianKinematics {
        steps_per_mm_x: config.stepper_x.steps_per_mm,
        steps_per_mm_y: config.stepper_y.steps_per_mm,
        steps_per_mm_z: config.stepper_z.steps_per_mm,
    };

    // Open the G-code file.
    let file = File::open(&args.gcode_file)
        .with_context(|| format!("Failed to open G-code file: {:?}", args.gcode_file))?;
    let reader = BufReader::new(file);

    // Simulate the printer's position.
    let mut current_pos = Position::default();

    // Process each line of the file.
    for (line_num, line_result) in reader.lines().enumerate() {
        let line = line_result?;
        if let Some(gcode) = parse_gcode(&line) {
            println!("\n[L{}] G-code: {:?}", line_num + 1, gcode);

            // Handle G0/G1 moves to calculate steps.
            if gcode.command == "G0" || gcode.command == "G1" {
                let mut new_pos = current_pos.clone();
                for (param, value) in gcode.params {
                    match param {
                        'X' => new_pos.x = value,
                        'Y' => new_pos.y = value,
                        'Z' => new_pos.z = value,
                        _ => {}
                    }
                }
                let steps = kinematics.calculate_move(&current_pos, &new_pos);
                println!("  -> MCU Steps: {:?}", steps);
                current_pos = new_pos;
            } else if gcode.command == "G28" {
                println!("  -> MCU Command: Home");
                current_pos = Position::default(); // Reset position after homing.
            }
        }
    }

    info!("Batch processing finished.");
    Ok(())
}

