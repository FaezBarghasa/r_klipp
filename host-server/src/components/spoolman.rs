//! Spoolman Client Component.
//! Connects Moonraker and Touch UI to the Spoolman filament management database.

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpoolInfo {
    pub id: u64,
    pub vendor: String,
    pub material: String,
    pub color_hex: String,
    pub remaining_weight_g: f32,
    pub total_weight_g: f32,
}

#[derive(Clone, Default)]
pub struct SpoolmanClient {
    active_spool: Arc<RwLock<Option<SpoolInfo>>>,
}

impl SpoolmanClient {
    pub fn new() -> Self {
        Self {
            active_spool: Arc::new(RwLock::new(Some(SpoolInfo {
                id: 1,
                vendor: "Prusament".to_string(),
                material: "PLA".to_string(),
                color_hex: "#FF5722".to_string(),
                remaining_weight_g: 742.5,
                total_weight_g: 1000.0,
            }))),
        }
    }

    pub async fn get_active_spool(&self) -> Option<SpoolInfo> {
        self.active_spool.read().await.clone()
    }

    pub async fn set_active_spool(&self, spool: Option<SpoolInfo>) {
        let mut active = self.active_spool.write().await;
        *active = spool;
    }
}
