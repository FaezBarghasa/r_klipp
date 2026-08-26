use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HeaterState {
    pub name: String,
    pub current: f32,
    pub target: f32,
    pub power: f32,
    pub history: Vec<f32>,
}

impl Default for HeaterState {
    fn default() -> Self {
        Self {
            name: "Extruder".to_string(),
            current: 215.0,
            target: 220.0,
            power: 0.45,
            history: vec![200.0, 205.0, 210.0, 212.0, 214.5, 215.0],
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ToolheadState {
    pub x: f32,
    pub y: f32,
    pub z: f32,
    pub e: f32,
    pub feedrate: f32,
    pub speed_factor: f32,
    pub extrude_factor: f32,
    pub fan_speed: f32,
    pub homed_axes: String,
}

impl Default for ToolheadState {
    fn default() -> Self {
        Self {
            x: 125.0,
            y: 125.0,
            z: 14.5,
            e: 142.3,
            feedrate: 150.0,
            speed_factor: 100.0,
            extrude_factor: 100.0,
            fan_speed: 80.0,
            homed_axes: "xyz".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PrintJobState {
    pub filename: String,
    pub progress: f32,
    pub print_time: u64,
    pub print_time_left: u64,
    pub total_layers: u32,
    pub current_layer: u32,
    pub filament_used_m: f32,
    pub status: String, // "printing", "paused", "standby", "complete", "error"
}

impl Default for PrintJobState {
    fn default() -> Self {
        Self {
            filename: "Voron_Cube_V2_PETG.gcode".to_string(),
            progress: 68.4,
            print_time: 2450,
            print_time_left: 1130,
            total_layers: 240,
            current_layer: 164,
            filament_used_m: 12.8,
            status: "printing".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SpoolInfo {
    pub id: u32,
    pub name: String,
    pub material: String,
    pub color_hex: String,
    pub remaining_weight_g: f32,
    pub total_weight_g: f32,
    pub temperature_nozzle: u32,
    pub temperature_bed: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MmuGate {
    pub gate_id: u8,
    pub filament_name: String,
    pub color_hex: String,
    pub is_loaded: bool,
    pub status: String, // "Pre-loaded", "In Toolhead", "Empty", "Error"
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AfcLane {
    pub lane_id: u8,
    pub spool_name: String,
    pub material: String,
    pub color_hex: String,
    pub status: String, // "Ready", "Feeding", "Hub Active", "Empty"
    pub buffer_length_mm: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MacroItem {
    pub name: String,
    pub category: String,
    pub description: String,
    pub is_running: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ConsoleEntry {
    pub id: u64,
    pub timestamp: String,
    pub message: String,
    pub is_command: bool,
    pub is_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct FarmPrinter {
    pub id: String,
    pub name: String,
    pub ip_address: String,
    pub status: String,
    pub progress: f32,
    pub current_file: String,
    pub nozzle_temp: f32,
    pub bed_temp: f32,
}
