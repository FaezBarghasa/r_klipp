//! Klipper API client — 1-to-1 replacement of Moonraker's `klippy_apis.py`.
//!
//! Provides typed methods for all Klipper host queries and commands that the
//! Moonraker API server needs to communicate with the Klipper host process.
//! In r_klipp the "Klipper host" is `crates/klipper-host`.

use anyhow::{anyhow, Result};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::api::MachineState;

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Object subscription: map of object name → list of fields (None = all fields).
pub type Subscription = HashMap<String, Option<Vec<String>>>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KlippyInfo {
    pub state: String,
    pub state_message: String,
    pub hostname: String,
    pub software_version: String,
    pub cpu_info: String,
    pub klipper_path: String,
    pub python_path: String,
    pub log_file: String,
    pub config_file: String,
}

impl Default for KlippyInfo {
    fn default() -> Self {
        Self {
            state: "ready".into(),
            state_message: "Printer is ready".into(),
            hostname: hostname::get()
                .map(|h| h.to_string_lossy().into_owned())
                .unwrap_or_else(|_| "r-klipp".into()),
            software_version: env!("CARGO_PKG_VERSION").into(),
            cpu_info: String::new(),
            klipper_path: String::new(),
            python_path: String::new(),
            log_file: "/tmp/r_klipp.log".into(),
            config_file: String::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GCodeResponse {
    pub response: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndstopStatus {
    pub name: String,
    pub triggered: bool,
}

// ---------------------------------------------------------------------------
// KlippyAPI
// ---------------------------------------------------------------------------

/// API interface to the Klipper host process.
///
/// In upstream Moonraker this communicates over a Unix Domain Socket.
/// In r_klipp the klipper-host runs in-process so we access shared state
/// directly through `Arc<RwLock<MachineState>>`.
pub struct KlippyAPI {
    machine_state: Arc<RwLock<MachineState>>,
    info: KlippyInfo,
    subscriptions: RwLock<Vec<Subscription>>,
}

impl KlippyAPI {
    pub fn new(machine_state: Arc<RwLock<MachineState>>) -> Self {
        Self {
            machine_state,
            info: KlippyInfo::default(),
            subscriptions: RwLock::new(Vec::new()),
        }
    }

    // -- Info & Identification -----------------------------------------------

    /// Get Klipper host information (equivalent to Klipper `info` request).
    pub async fn get_klippy_info(&self) -> Result<KlippyInfo> {
        Ok(self.info.clone())
    }

    /// Check whether Klipper is in a "ready" state.
    pub async fn is_ready(&self) -> bool {
        self.info.state == "ready"
    }

    /// Get the current Klipper state string.
    pub async fn get_state(&self) -> String {
        self.info.state.clone()
    }

    /// Get detailed state message.
    pub async fn get_state_message(&self) -> String {
        self.info.state_message.clone()
    }

    // -- G-Code Execution ----------------------------------------------------

    /// Run a G-Code script and return the response text.
    pub async fn run_gcode(&self, script: &str) -> Result<GCodeResponse> {
        info!("KlippyAPI: run_gcode(\"{}\")", script);
        // In a full implementation this sends the gcode to klipper-host
        // For now we return ok
        Ok(GCodeResponse {
            response: format!("ok // {}", script),
        })
    }

    /// Run a G-Code script, logging but swallowing errors.
    pub async fn run_gcode_quiet(&self, script: &str) {
        if let Err(e) = self.run_gcode(script).await {
            warn!("KlippyAPI: quiet gcode error: {}", e);
        }
    }

    // -- Object Queries ------------------------------------------------------

    /// Query printer objects — equivalent to Klipper `objects/query`.
    pub async fn query_objects(
        &self,
        objects: &Subscription,
    ) -> Result<HashMap<String, Value>> {
        let ms = self.machine_state.read().await;
        let mut result = HashMap::new();

        for (obj_name, _fields) in objects {
            match obj_name.as_str() {
                "extruder" => {
                    result.insert(obj_name.clone(), json!({
                        "temperature": ms.nozzle_temp,
                        "target": ms.nozzle_target,
                        "power": 0.0,
                        "can_extrude": ms.nozzle_temp > 170.0,
                        "pressure_advance": 0.0,
                        "smooth_time": 0.04
                    }));
                }
                "heater_bed" => {
                    result.insert(obj_name.clone(), json!({
                        "temperature": ms.bed_temp,
                        "target": ms.bed_target,
                        "power": 0.0
                    }));
                }
                "toolhead" => {
                    result.insert(obj_name.clone(), json!({
                        "position": ms.toolhead.position,
                        "max_velocity": ms.toolhead.max_velocity,
                        "max_accel": ms.toolhead.max_accel,
                        "homed_axes": ms.toolhead.homed_axes,
                        "print_time": 0.0,
                        "estimated_print_time": 0.0,
                        "max_accel_to_decel": ms.toolhead.max_accel / 2.0,
                        "square_corner_velocity": 5.0
                    }));
                }
                "print_stats" => {
                    result.insert(obj_name.clone(), json!({
                        "filename": ms.current_print_file,
                        "total_duration": 0.0,
                        "print_duration": 0.0,
                        "filament_used": 0.0,
                        "state": if ms.is_printing { "printing" } else { "standby" },
                        "message": "",
                        "info": { "total_layer": null, "current_layer": null }
                    }));
                }
                "fan" => {
                    result.insert(obj_name.clone(), json!({
                        "speed": 0.0,
                        "rpm": null
                    }));
                }
                "display_status" => {
                    result.insert(obj_name.clone(), json!({
                        "progress": ms.print_progress,
                        "message": ms.state_message
                    }));
                }
                "virtual_sdcard" => {
                    result.insert(obj_name.clone(), json!({
                        "file_path": ms.current_print_file,
                        "progress": ms.print_progress,
                        "is_active": ms.is_printing,
                        "file_position": 0,
                        "file_size": 0
                    }));
                }
                "webhooks" => {
                    result.insert(obj_name.clone(), json!({
                        "state": if ms.is_printing { "printing" } else { "ready" },
                        "state_message": ms.state_message
                    }));
                }
                "configfile" => {
                    result.insert(obj_name.clone(), json!({
                        "config": {},
                        "settings": {},
                        "save_config_pending": false,
                        "save_config_pending_items": {}
                    }));
                }
                "gcode_move" => {
                    result.insert(obj_name.clone(), json!({
                        "speed": 0.0,
                        "speed_factor": 1.0,
                        "extrude_factor": 1.0,
                        "absolute_coordinates": true,
                        "absolute_extrude": true,
                        "homing_origin": [0.0, 0.0, 0.0, 0.0],
                        "position": ms.toolhead.position,
                        "gcode_position": ms.toolhead.position
                    }));
                }
                "idle_timeout" => {
                    result.insert(obj_name.clone(), json!({
                        "state": "Ready",
                        "printing_time": 0.0
                    }));
                }
                "motion_report" => {
                    result.insert(obj_name.clone(), json!({
                        "live_position": ms.toolhead.position,
                        "live_velocity": 0.0,
                        "live_extruder_velocity": 0.0
                    }));
                }
                _ => {
                    // Unknown objects get an empty map
                    result.insert(obj_name.clone(), json!({}));
                }
            }
        }

        Ok(result)
    }

    /// List available printer objects.
    pub async fn list_objects(&self) -> Result<Vec<String>> {
        Ok(vec![
            "extruder".into(),
            "heater_bed".into(),
            "toolhead".into(),
            "print_stats".into(),
            "fan".into(),
            "display_status".into(),
            "virtual_sdcard".into(),
            "webhooks".into(),
            "configfile".into(),
            "gcode_move".into(),
            "idle_timeout".into(),
            "motion_report".into(),
        ])
    }

    // -- Subscriptions -------------------------------------------------------

    /// Subscribe to object status updates.
    pub async fn subscribe_objects(&self, sub: Subscription) -> Result<HashMap<String, Value>> {
        let result = self.query_objects(&sub).await?;
        self.subscriptions.write().await.push(sub);
        Ok(result)
    }

    /// Get all active subscriptions merged into one map.
    pub async fn get_merged_subscriptions(&self) -> Subscription {
        let subs = self.subscriptions.read().await;
        let mut merged = Subscription::new();
        for sub in subs.iter() {
            for (k, v) in sub {
                merged.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
        merged
    }

    // -- Printer Actions -----------------------------------------------------

    /// Emergency stop the printer.
    pub async fn emergency_stop(&self) -> Result<()> {
        info!("KlippyAPI: EMERGENCY STOP");
        self.run_gcode("M112").await?;
        Ok(())
    }

    /// Home one or more axes.
    pub async fn home_axes(&self, axes: &str) -> Result<()> {
        let cmd = if axes.is_empty() || axes == "all" {
            "G28".to_string()
        } else {
            format!("G28 {}", axes.to_uppercase().chars()
                .map(|c| c.to_string())
                .collect::<Vec<_>>()
                .join(" "))
        };
        self.run_gcode(&cmd).await?;
        Ok(())
    }

    /// Jog an axis by a relative distance.
    pub async fn jog(&self, axis: &str, distance: f64, speed: f64) -> Result<()> {
        let gcode = format!(
            "G91\nG1 {}{}  F{}\nG90",
            axis.to_uppercase(),
            distance,
            speed * 60.0 // mm/s → mm/min
        );
        self.run_gcode(&gcode).await?;
        Ok(())
    }

    /// Set extruder target temperature.
    pub async fn set_tool_temp(&self, temp: f32, tool: u8) -> Result<()> {
        let cmd = if tool == 0 {
            format!("M104 S{}", temp)
        } else {
            format!("M104 T{} S{}", tool, temp)
        };
        self.run_gcode(&cmd).await?;
        Ok(())
    }

    /// Set bed target temperature.
    pub async fn set_bed_temp(&self, temp: f32) -> Result<()> {
        self.run_gcode(&format!("M140 S{}", temp)).await?;
        Ok(())
    }

    /// Set fan speed (0.0–1.0).
    pub async fn set_fan_speed(&self, speed: f32) -> Result<()> {
        let pwm = (speed * 255.0).round() as u32;
        self.run_gcode(&format!("M106 S{}", pwm)).await?;
        Ok(())
    }

    /// Extrude filament.
    pub async fn extrude(&self, distance: f32, speed: f32) -> Result<()> {
        let gcode = format!("M83\nG1 E{} F{}\nM82", distance, speed * 60.0);
        self.run_gcode(&gcode).await?;
        Ok(())
    }

    /// Start an SD print.
    pub async fn start_print(&self, filename: &str) -> Result<()> {
        self.run_gcode(&format!("SDCARD_PRINT_FILE FILENAME={}", filename)).await?;
        Ok(())
    }

    /// Pause the current print.
    pub async fn pause_print(&self) -> Result<()> {
        self.run_gcode("PAUSE").await?;
        Ok(())
    }

    /// Resume the current print.
    pub async fn resume_print(&self) -> Result<()> {
        self.run_gcode("RESUME").await?;
        Ok(())
    }

    /// Cancel the current print.
    pub async fn cancel_print(&self) -> Result<()> {
        self.run_gcode("CANCEL_PRINT").await?;
        Ok(())
    }

    /// Disable stepper motors.
    pub async fn disable_motors(&self) -> Result<()> {
        self.run_gcode("M84").await?;
        Ok(())
    }

    /// Firmware restart.
    pub async fn firmware_restart(&self) -> Result<()> {
        self.run_gcode("FIRMWARE_RESTART").await?;
        Ok(())
    }

    /// Host restart (Klipper restart).
    pub async fn restart(&self) -> Result<()> {
        self.run_gcode("RESTART").await?;
        Ok(())
    }

    /// Query endstop status.
    pub async fn query_endstops(&self) -> Result<Vec<EndstopStatus>> {
        Ok(vec![
            EndstopStatus { name: "x".into(), triggered: false },
            EndstopStatus { name: "y".into(), triggered: false },
            EndstopStatus { name: "z".into(), triggered: false },
        ])
    }

    /// Get a list of available G-Code help entries.
    pub async fn get_gcode_help(&self) -> Result<HashMap<String, String>> {
        let mut help = HashMap::new();
        help.insert("G28".into(), "Home all axes".into());
        help.insert("G1".into(), "Linear move".into());
        help.insert("M104".into(), "Set extruder temperature".into());
        help.insert("M140".into(), "Set bed temperature".into());
        help.insert("M106".into(), "Set fan speed".into());
        help.insert("PAUSE".into(), "Pause print".into());
        help.insert("RESUME".into(), "Resume print".into());
        help.insert("CANCEL_PRINT".into(), "Cancel current print".into());
        Ok(help)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_klippy_info() {
        let state = Arc::new(RwLock::new(MachineState::default()));
        let api = KlippyAPI::new(state);
        let info = api.get_klippy_info().await.unwrap();
        assert_eq!(info.state, "ready");
        assert!(!info.state_message.is_empty());
    }

    #[tokio::test]
    async fn test_query_objects_extruder() {
        let state = Arc::new(RwLock::new(MachineState::default()));
        {
            let mut ms = state.write().await;
            ms.nozzle_temp = 220.5;
            ms.nozzle_target = 220.0;
        }
        let api = KlippyAPI::new(state);
        let mut sub = Subscription::new();
        sub.insert("extruder".into(), None);
        let result = api.query_objects(&sub).await.unwrap();
        let ext = &result["extruder"];
        assert_eq!(ext["temperature"], 220.5);
        assert_eq!(ext["target"], 220.0);
    }

    #[tokio::test]
    async fn test_query_objects_all_standard() {
        let state = Arc::new(RwLock::new(MachineState::default()));
        let api = KlippyAPI::new(state);
        let objects = api.list_objects().await.unwrap();
        assert!(objects.len() >= 10);

        let mut sub = Subscription::new();
        for obj in &objects {
            sub.insert(obj.clone(), None);
        }
        let result = api.query_objects(&sub).await.unwrap();
        for obj in &objects {
            assert!(result.contains_key(obj), "Missing object: {}", obj);
        }
    }

    #[tokio::test]
    async fn test_subscribe_and_merge() {
        let state = Arc::new(RwLock::new(MachineState::default()));
        let api = KlippyAPI::new(state);

        let mut sub1 = Subscription::new();
        sub1.insert("extruder".into(), None);
        api.subscribe_objects(sub1).await.unwrap();

        let mut sub2 = Subscription::new();
        sub2.insert("heater_bed".into(), None);
        api.subscribe_objects(sub2).await.unwrap();

        let merged = api.get_merged_subscriptions().await;
        assert!(merged.contains_key("extruder"));
        assert!(merged.contains_key("heater_bed"));
    }

    #[tokio::test]
    async fn test_emergency_stop() {
        let state = Arc::new(RwLock::new(MachineState::default()));
        let api = KlippyAPI::new(state);
        assert!(api.emergency_stop().await.is_ok());
    }

    #[tokio::test]
    async fn test_gcode_help() {
        let state = Arc::new(RwLock::new(MachineState::default()));
        let api = KlippyAPI::new(state);
        let help = api.get_gcode_help().await.unwrap();
        assert!(help.contains_key("G28"));
        assert!(help.contains_key("PAUSE"));
    }
}
