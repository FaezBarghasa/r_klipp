//! Moonraker Update Manager Component.
//! Tracks software versions and repository status.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComponentVersionInfo {
    pub name: String,
    pub branch: String,
    pub remote_alias: String,
    pub version: String,
    pub commit_hash: String,
    pub is_dirty: bool,
    pub configured_type: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateStatus {
    pub version_info: std::collections::HashMap<String, ComponentVersionInfo>,
    pub busy: bool,
}

#[derive(Clone, Default)]
pub struct UpdateManager;

impl UpdateManager {
    pub fn new() -> Self {
        Self
    }

    pub async fn get_status(&self) -> UpdateStatus {
        let mut map = std::collections::HashMap::new();
        map.insert("r_klipp".to_string(), ComponentVersionInfo {
            name: "r_klipp".to_string(),
            branch: "main".to_string(),
            remote_alias: "origin".to_string(),
            version: "0.1.0-rklipp".to_string(),
            commit_hash: "9a2f4c1".to_string(),
            is_dirty: false,
            configured_type: "git_repo".to_string(),
        });
        map.insert("fluidd".to_string(), ComponentVersionInfo {
            name: "fluidd".to_string(),
            branch: "main".to_string(),
            remote_alias: "origin".to_string(),
            version: "v1.31.0".to_string(),
            commit_hash: "f83a210".to_string(),
            is_dirty: false,
            configured_type: "web".to_string(),
        });
        map.insert("KlipperScreen".to_string(), ComponentVersionInfo {
            name: "KlipperScreen".to_string(),
            branch: "master".to_string(),
            remote_alias: "origin".to_string(),
            version: "v0.4.5".to_string(),
            commit_hash: "7bc3291".to_string(),
            is_dirty: false,
            configured_type: "git_repo".to_string(),
        });

        UpdateStatus {
            version_info: map,
            busy: false,
        }
    }
}
