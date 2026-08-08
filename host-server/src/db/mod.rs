pub mod models;

use anyhow::Result;
use surrealdb::{
    engine::local::{Db, File},
    Surreal,
};
use thiserror::Error;

use crate::db::models::{GCodeFile, MachineConfig, PrintHistory};

#[derive(Error, Debug)]
pub enum HostError {
    #[error("Database error: {0}")]
    DbError(#[from] surrealdb::Error),
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),
    #[error("Other error: {0}")]
    Other(String),
}

pub struct Database {
    db: Surreal<Db>,
}

impl Database {
    pub async fn new(path: &str) -> Result<Self, HostError> {
        let db = Surreal::new::<File>(path).await?;
        db.use_ns("r_klipp").use_db("host_data").await?;
        Ok(Self { db })
    }

    pub async fn init_schema(&self) -> Result<(), HostError> {
        // Define tables and indexes
        // MachineConfig table
        self.db
            .query("DEFINE TABLE machine_config SCHEMAFULL;")
            .await?;
        self.db
            .query("DEFINE FIELD name ON TABLE machine_config TYPE string;")
            .await?;
        self.db
            .query("DEFINE FIELD config ON TABLE machine_config TYPE object;")
            .await?;

        // GCodeFile table
        self.db
            .query("DEFINE TABLE gcode_file SCHEMAFULL;")
            .await?;
        self.db
            .query("DEFINE FIELD path ON TABLE gcode_file TYPE string;")
            .await?;
        self.db
            .query("DEFINE FIELD size ON TABLE gcode_file TYPE int;")
            .await?;
        self.db
            .query("DEFINE FIELD upload_date ON TABLE gcode_file TYPE datetime;")
            .await?;
        self.db
            .query("DEFINE FIELD metadata ON TABLE gcode_file TYPE object;")
            .await?;
        self.db
            .query("DEFINE INDEX unique_path ON TABLE gcode_file COLUMNS path UNIQUE;")
            .await?;

        // PrintHistory table
        self.db
            .query("DEFINE TABLE print_history SCHEMAFULL;")
            .await?;
        self.db
            .query("DEFINE FIELD start_time ON TABLE print_history TYPE datetime;")
            .await?;
        self.db
            .query("DEFINE FIELD end_time ON TABLE print_history TYPE datetime;")
            .await?;
        self.db
            .query("DEFINE FIELD status ON TABLE print_history TYPE string;")
            .await?;
        self.db
            .query("DEFINE FIELD telemetry_summary ON TABLE print_history TYPE object;")
            .await?;

        Ok(())
    }

    pub async fn save_gcode_metadata(&self, meta: GCodeFile) -> Result<(), HostError> {
        let _created: GCodeFile = self
            .db
            .create("gcode_file")
            .content(meta)
            .await?;
        Ok(())
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
            .take(0)?;
        Ok(history)
    }

    pub async fn save_print_history(&self, history: PrintHistory) -> Result<(), HostError> {
        let _created: PrintHistory = self
            .db
            .create("print_history")
            .content(history)
            .await?;
        Ok(())
    }

    pub async fn save_machine_config(&self, config: MachineConfig) -> Result<(), HostError> {
        let _created: MachineConfig = self
            .db
            .create("machine_config")
            .content(config)
            .await?;
        Ok(())
    }

    pub async fn get_machine_config(&self, name: &str) -> Result<Option<MachineConfig>, HostError> {
        let config: Option<MachineConfig> = self
            .db
            .select(("machine_config", name))
            .await?;
        Ok(config)
    }
}
