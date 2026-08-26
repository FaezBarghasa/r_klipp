use serde_json::json;

pub const DEFAULT_MOONRAKER_WS_URL: &str = "ws://127.0.0.1:7125/websocket";
pub const DEFAULT_MOONRAKER_HTTP_URL: &str = "http://127.0.0.1:7125";

pub fn create_subscribe_payload() -> String {
    json!({
        "jsonrpc": "2.0",
        "method": "printer.objects.subscribe",
        "params": {
            "objects": {
                "extruder": ["temperature", "target", "power"],
                "heater_bed": ["temperature", "target", "power"],
                "toolhead": ["position", "max_velocity", "max_accel", "homed_axes"],
                "print_stats": ["filename", "progress", "state", "total_duration", "print_duration", "filament_used"],
                "mmu": ["gate", "status", "enabled"],
                "afc": ["lane", "status", "buffer"],
                "spoolman": ["active_spool"]
            }
        },
        "id": 1
    }).to_string()
}
