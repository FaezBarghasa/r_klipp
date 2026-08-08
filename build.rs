use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;

// This is a placeholder for the actual config struct.
// In a real build script, you'd likely have a shared crate for the schema.
#[derive(serde::Deserialize)]
struct RklippConfig {
    machine_profile: String, // Using String for simplicity here
    mcu_selection: McuSelection,
    axes: Vec<AxisConfig>,
    pin_mapping: PinMapping,
}
#[derive(serde::Deserialize)]
struct McuSelection {
    target_triple: String,
    memory_x_script: String,
}
#[derive(serde::Deserialize)]
struct AxisConfig {
    // fields...
}
#[derive(serde::Deserialize)]
struct PinMapping {
    pins: std::collections::HashMap<String, String>,
}


fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("config.rs");

    let config_path = Path::new("../.rklipp_config.toml");

    if !config_path.exists() {
        panic!("Configuration file .rklipp_config.toml not found. Please run the configuration tool first.");
    }

    let toml_str = fs::read_to_string(config_path).unwrap();
    let config: RklippConfig = toml::from_str(&toml_str).unwrap();

    let mut generated_code = String::new();

    // Generate feature flags
    let feature = match config.machine_profile.as_str() {
        "FdmPrinter" => "fdm_dialect",
        "CncRouter" => "cnc_dialect",
        "FiveAxisCnc" => "five_axis_dialect",
        "WireEdm" => "wire_edm_dialect",
        "PickAndPlace" => "pnp_dialect",
        _ => "fdm_dialect", // default
    };
    println!("cargo:rustc-cfg=feature=\"{}\"", feature);

    // Set linker script
    let memory_script = &config.mcu_selection.memory_x_script;
    println!("cargo:rustc-link-arg=-T{}", memory_script);
    println!("cargo:rerun-if-changed={}", memory_script);


    // Generate constants
    generated_code.push_str(&format!("pub const AXIS_COUNT: usize = {};\n", config.axes.len()));
    generated_code.push_str(&format!("pub const MCU_TARGET_TRIPLE: &str = \"{}\";\n", config.mcu_selection.target_triple));

    // Generate pin mappings
    for (name, pin) in &config.pin_mapping.pins {
        // This is a simplified representation. A real implementation would parse the pin string
        // into a port and pin number.
        generated_code.push_str(&format!("pub const PIN_{}: &str = \"{}\";\n", name.to_uppercase(), pin));
    }


    fs::write(&dest_path, generated_code).unwrap();
    println!("cargo:rerun-if-changed=../.rklipp_config.toml");
}
