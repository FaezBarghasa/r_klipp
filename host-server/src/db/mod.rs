use anyhow::Result;
use serde::{Deserialize, Serialize};
use surrealdb::{
    engine::local::{Db, File},
    Surreal,
};
use thiserror::Error;
use chrono::{DateTime, Utc};

#[derive(Error, Debug)]
pub enum HostError {
    #[error("Database error: {0}")]
    Database(#[from] surrealdb::Error),
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}

// Define the schema and Rust models
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MachineConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub axis_limits: AxisLimits,
    pub pid_gains: PidGains,
    pub pin_mappings: PinMappings,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AxisLimits {
    pub x_max: f32,
    pub y_max: f32,
    pub z_max: f32,
    pub x_min: f32,
    pub y_min: f32,
    pub z_min: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PidGains {
    pub kp: f32,
    pub ki: f32,
    pub kd: f32,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PinMappings {
    pub heater_bed: String,
    pub heater_nozzle: String,
    pub therm_bed: String,
    pub therm_nozzle: String,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct GCodeFile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub path: String,
    pub name: String,
    pub size: u64,
    pub upload_date: DateTime<Utc>,
    pub estimated_time_secs: Option<u32>,
    pub thumbnail_path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PrintHistory {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub file_id: String, // Link to GCodeFile
    pub start_time: DateTime<Utc>,
    pub end_time: Option<DateTime<Utc>>,
    pub status: String, // "Completed", "Failed", "Cancelled"
    pub duration_secs: Option<u32>,
    pub telemetry_summary: Option<serde_json::Value>, // e.g., max temps, avg power
}

#[derive(Clone)]
pub struct Database {
    db: Surreal<Db>,
}

impl Database {
    pub async fn new() -> Result<Self, HostError> {
        let db = Surreal::new::<File>("./r_klipp_data/db.db").await?;
        db.use_ns("r_klipp").use_db("host").await?;
        Ok(Database { db })
    }

    pub async fn init_schema(&self) -> Result<(), HostError> {
        // Define tables and indexes
        self.db
            .query("DEFINE TABLE machine_config SCHEMAFULL;")
            .await?
            .check()?;
        self.db
            .query("DEFINE FIELD axis_limits ON TABLE machine_config TYPE object;")
            .await?
            .check()?;
        self.db
            .query("DEFINE FIELD pid_gains ON TABLE machine_config TYPE object;")
            .await?
            .check()?;
        self.db
            .query("DEFINE FIELD pin_mappings ON TABLE machine_config TYPE object;")
            .await?
            .check()?;

        self.db
            .query("DEFINE TABLE gcode_file SCHEMAFULL;")
            .await?
            .check()?;
        self.db
            .query("DEFINE FIELD path ON TABLE gcode_file TYPE string ASSERT $value != NONE;")
            .await?
            .check()?;
        self.db
            .query("DEFINE FIELD name ON TABLE gcode_file TYPE string ASSERT $value != NONE;")
            .await?
            .check()?;
        self.db
            .query("DEFINE FIELD size ON TABLE gcode_file TYPE int ASSERT $value >= 0;")
            .await?
            .check()?;
        self.db
            .query("DEFINE FIELD upload_date ON TABLE gcode_file TYPE datetime DEFAULT time::now();")
            .await?
            .check()?;

        self.db
            .query("DEFINE TABLE print_history SCHEMAFULL;")
            .await?
            .check()?;
        self.db
            .query("DEFINE FIELD file_id ON TABLE print_history TYPE record(gcode_file) ASSERT $value != NONE;")
            .await?
            .check()?;
        self.db
            .query("DEFINE FIELD start_time ON TABLE print_history TYPE datetime DEFAULT time::now();")
            .await?
            .check()?;
        self.db
            .query("DEFINE FIELD status ON TABLE print_history TYPE string ASSERT $value INSIDE ['Completed', 'Failed', 'Cancelled', 'Printing'];")
            .await?
            .check()?;

        Ok(())
    }

    pub async fn save_gcode_metadata(&self, meta: GCodeFile) -> Result<GCodeFile, HostError> {
        let created: Vec<GCodeFile> = self
            .db
            .create("gcode_file")
            .content(meta)
            .await?;
        created.first().cloned().ok_or_else(|| HostError::Other("Failed to save GCodeFile metadata".into()))
    }

    pub async fn get_gcode_files(&self) -> Result<Vec<GCodeFile>, HostError> {
        let files: Vec<GCodeFile> = self.db.select("gcode_file").await?;
        Ok(files)
    }

    pub async fn get_print_history(&self, limit: u32) -> Result<Vec<PrintHistory>, HostError> {
        let history: Vec<PrintHistory> = self
            .db
            .query(format!("SELECT * FROM print_history ORDER BY start_time DESC LIMIT {};", limit))
            .await?
            .take(0)
            .map_err(|e| HostError::Database(e))?;
        Ok(history)
    }

    pub async fn create_print_history(&self, entry: PrintHistory) -> Result<PrintHistory, HostError> {
        let created: Vec<PrintHistory> = self
            .db
            .create("print_history")
            .content(entry)
            .await?;
        created.first().cloned().ok_or_else(|| HostError::Other("Failed to create print history entry".into()))
    }

    pub async fn update_print_history(&self, id: String, updates: serde_json::Value) -> Result<PrintHistory, HostError> {
        let updated: Vec<PrintHistory> = self
            .db
            .update(id)
            .merge(updates)
            .await?;
        updated.first().cloned().ok_or_else(|| HostError::Other("Failed to update print history entry".into()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::fs;

    async fn setup_test_db() -> Database {
        let db_path = "./r_klipp_data_test/db.db";
        // Ensure a clean state for each test
        let _ = fs::remove_dir_all("./r_klipp_data_test").await;
        let db = Surreal::new::<File>(db_path).await.unwrap();
        db.use_ns("r_klipp_test").use_db("host_test").await.unwrap();
        let database = Database { db };
        database.init_schema().await.unwrap();
        database
    }

    #[tokio::test]
    async fn test_db_init_and_config_save() {
        let db = setup_test_db().await;

        let config = MachineConfig {
            id: None,
            axis_limits: AxisLimits {
                x_max: 200.0,
                y_max: 200.0,
                z_max: 200.0,
                x_min: 0.0,
                y_min: 0.0,
                z_min: 0.0,
            },
            pid_gains: PidGains {
                kp: 22.2,
                ki: 1.08,
                kd: 114.0,
            },
            pin_mappings: PinMappings {
                heater_bed: "PB0".into(),
                heater_nozzle: "PA0".into(),
                therm_bed: "PC0".into(),
                therm_nozzle: "PD0".into(),
            },
        };

        let created: Vec<MachineConfig> = db.db.create("machine_config").content(config).await.unwrap();
        assert!(!created.is_empty());
        assert!(created[0].id.is_some());
    }

    #[tokio::test]
    async fn test_gcode_file_save_and_retrieve() {
        let db = setup_test_db().await;

        let file = GCodeFile {
            id: None,
            path: "/path/to/test.gcode".into(),
            name: "test.gcode".into(),
            size: 1024,
            upload_date: Utc::now(),
            estimated_time_secs: Some(3600),
            thumbnail_path: None,
        };

        let saved_file = db.save_gcode_metadata(file.clone()).await.unwrap();
        assert!(saved_file.id.is_some());
        assert_eq!(saved_file.name, "test.gcode");

        let retrieved_files = db.get_gcode_files().await.unwrap();
        assert!(!retrieved_files.is_empty());
        assert_eq!(retrieved_files[0].name, "test.gcode");
    }

    #[tokio::test]
    async fn test_print_history_create_and_retrieve() {
        let db = setup_test_db().await;

        let file = GCodeFile {
            id: None,
            path: "/path/to/test_print.gcode".into(),
            name: "test_print.gcode".into(),
            size: 2048,
            upload_date: Utc::now(),
            estimated_time_secs: Some(7200),
            thumbnail_path: None,
        };
        let saved_file = db.save_gcode_metadata(file).await.unwrap();

        let history_entry = PrintHistory {
            id: None,
            file_id: saved_file.id.unwrap(),
            start_time: Utc::now(),
            end_time: None,
            status: "Printing".into(),
            duration_secs: None,
            telemetry_summary: None,
        };

        let created_history = db.create_print_history(history_entry.clone()).await.unwrap();
        assert!(created_history.id.is_some());
        assert_eq!(created_history.status, "Printing");

        let retrieved_history = db.get_print_history(10).await.unwrap();
        assert!(!retrieved_history.is_empty());
        assert_eq!(retrieved_history[0].status, "Printing");

        let updates = serde_json::json!({
            "status": "Completed",
            "end_time": Utc::now(),
            "duration_secs": 7200,
        });
        let updated_history = db.update_print_history(created_history.id.unwrap(), updates).await.unwrap();
        assert_eq!(updated_history.status, "Completed");
        assert!(updated_history.end_time.is_some());
        assert!(updated_history.duration_secs.is_some());
    }
}