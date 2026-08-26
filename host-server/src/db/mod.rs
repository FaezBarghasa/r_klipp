pub mod models;

use anyhow::Result;
use surrealdb::{
    engine::local::{Db, SurrealKv},
    Surreal,
};
use thiserror::Error;

use crate::db::models::{GCodeFile, MachineConfig, PrintHistory};

#[derive(Error, Debug)]
#[allow(dead_code)]
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

#[allow(dead_code)]
impl Database {
    pub async fn new(path: &str) -> Result<Self, HostError> {
        let db = Surreal::new::<SurrealKv>(path).await?;
        db.use_ns("r_klipp").use_db("host_data").await?;
        Ok(Self { db })
    }

    pub async fn init_schema(&self) -> Result<(), HostError> {
        // MachineConfig table
        self.db
            .query("DEFINE TABLE machine_config SCHEMALESS;")
            .await?;

        // GCodeFile table
        self.db
            .query("DEFINE TABLE gcode_file SCHEMALESS;")
            .await?;

        // PrintHistory table
        self.db
            .query("DEFINE TABLE print_history SCHEMALESS;")
            .await?;

        Ok(())
    }

    pub async fn save_gcode_metadata(&self, meta: GCodeFile) -> Result<(), HostError> {
        let _created: Option<GCodeFile> = self
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
        let _created: Option<PrintHistory> = self
            .db
            .create("print_history")
            .content(history)
            .await?;
        Ok(())
    }

    pub async fn save_machine_config(&self, config: MachineConfig) -> Result<(), HostError> {
        let _created: Option<MachineConfig> = self
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
