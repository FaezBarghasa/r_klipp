use clap::{Parser, Subcommand};
use std::process::{Command, Stdio};
use std::fs;
use std::path::Path;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Launch the terminal UI to configure r_klipp
    Config,
    /// Build the r_klipp firmware
    Build,
    /// Flash the firmware to the target MCU
    Flash,
}

// Placeholder for the config struct
#[derive(serde::Deserialize)]
struct RklippConfig {
    machine_profile: String,
    mcu_selection: McuSelection,
}
#[derive(serde::Deserialize)]
struct McuSelection {
    target_triple: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Config => {
            println!("Launching configuration TUI...");
            let status = Command::new("cargo")
                .args(["run", "-p", "rklipp-config"])
                .status()?;
            if !status.success() {
                eprintln!("Configuration tool exited with an error.");
            }
        }
        Commands::Build => {
            println!("Building firmware...");
            let config_path = Path::new("../../.rklipp_config.toml");
            if !config_path.exists() {
                eprintln!("Configuration file not found. Run `rklipp-build config` first.");
                return Ok(());
            }
            let toml_str = fs::read_to_string(config_path)?;
            let config: RklippConfig = toml::from_str(&toml_str)?;

            let feature = match config.machine_profile.as_str() {
                "FdmPrinter" => "fdm_dialect",
                "CncRouter" => "cnc_dialect",
                "FiveAxisCnc" => "five_axis_dialect",
                "WireEdm" => "wire_edm_dialect",
                "PickAndPlace" => "pnp_dialect",
                _ => "fdm_dialect",
            };

            let build_status = Command::new("cargo")
                .args([
                    "build",
                    "--release",
                    "--target",
                    &config.mcu_selection.target_triple,
                    "--features",
                    feature,
                ])
                .current_dir("../../") // Run from the root of the firmware crate
                .status()?;

            if !build_status.success() {
                eprintln!("Build failed.");
                return Ok(());
            }

            println!("Build successful. Generating binary artifacts...");
            let target_triple = &config.mcu_selection.target_triple;
            let elf_path = format!("../../target/{}/release/r_klipp", target_triple);

            // Generate .bin
            let bin_path = format!("../../target/{}/release/r_klipp.bin", target_triple);
            let objcopy_status_bin = Command::new("rust-objcopy")
                .args(["-O", "binary", &elf_path, &bin_path])
                .status()?;

            if !objcopy_status_bin.success() {
                eprintln!("Failed to generate .bin file. Is `cargo-binutils` installed?");
            }

            // Generate .hex
            let hex_path = format!("../../target/{}/release/r_klipp.hex", target_triple);
            let objcopy_status_hex = Command::new("rust-objcopy")
                .args(["-O", "ihex", &elf_path, &hex_path])
                .status()?;

            if !objcopy_status_hex.success() {
                eprintln!("Failed to generate .hex file.");
            }

            println!("Artifacts generated in target/{}/release/", target_triple);
        }
        Commands::Flash => {
            println!("Flashing firmware... (not implemented)");
            // Example:
            // Command::new("probe-rs")
            //     .args(["run", "--chip", "STM32F407VE", "../../target/thumbv7em-none-eabihf/release/r_klipp"])
            //     .status()?;
        }
    }

    Ok(())
}
