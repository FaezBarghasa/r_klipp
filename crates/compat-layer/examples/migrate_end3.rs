use compat_layer::migrator::migrate_config;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("--- Running Ender 3 Config Migration Example ---");

    // We assume the example is run from the root of the `compat-layer` crate.
    let config_path = Path::new("tests/test_configs/ender3.cfg");
    if !config_path.exists() {
        eprintln!(
            "Error: Could not find sample config at '{}'.",
            config_path.display()
        );
        eprintln!("Please run this example from the crate's root directory via `cargo run --example migrate_end3`.");
        return Ok(());
    }

    let config_content = fs::read_to_string(config_path)?;

    println!("\n--- Migrating file: {} ---", config_path.display());
    let (config, report) = migrate_config(&config_content)?;

    println!("Migration successful!");

    if !report.warnings.is_empty() {
        println!("\n--- Migration Warnings ---");
        for warning in report.warnings {
            println!("- {}", warning);
        }
    }

    if !report.unsupported_sections.is_empty() {
        println!("\n--- Unsupported Sections (stored in `other_sections`) ---");
        for section in report.unsupported_sections {
            println!("- [{}]", section);
        }
    }

    let json_output = serde_json::to_string_pretty(&config)?;
    println!("\n--- Resulting Rust Config (as JSON) ---");
    println!("{}", json_output);

    Ok(())
}
