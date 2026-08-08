use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use std::collections::HashMap;

#[derive(Debug, Serialize, Deserialize)]
pub struct MachineConfig {
    pub name: String,
    pub config: HashMap<String, serde_json::Value>, // Store as generic JSON object
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GCodeFile {
    pub id: Option<surrealdb::sql::Thing>, // SurrealDB ID
    pub path: String,
    pub size: u64,
    pub upload_date: DateTime<Utc>,
    pub metadata: GCodeMetadata,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct GCodeMetadata {
    pub estimated_time: Option<u32>,
    pub layer_height: Option<f32>,
    pub filament_length: Option<f32>,
    pub thumbnails: Vec<Thumbnail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Thumbnail {
    pub width: u32,
    pub height: u32,
    pub data: String, // Base64 encoded image data
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PrintHistory {
    pub id: Option<surrealdb::sql::Thing>, // SurrealDB ID
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: PrintStatus,
    pub telemetry_summary: PrintTelemetrySummary,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PrintStatus {
    #[serde(rename = "in_progress")]
    InProgress,
    #[serde(rename = "completed")]
    Completed,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct PrintTelemetrySummary {
    pub max_nozzle_temp: Option<f32>,
    pub max_bed_temp: Option<f32>,
    pub total_filament_used: Option<f32>,
    // Add other relevant summary statistics
}
