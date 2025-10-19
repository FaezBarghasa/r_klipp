//! Dynamic mapping of command names to IDs and versioning helpers.
//!
//! Klipper does not use fixed IDs for its commands. Instead, the host and MCU
//! negotiate a mapping of command names (strings) to message IDs (bytes)
//! upon connection. This module provides a `CommandRegistry` to manage this
//! mapping.

#[cfg(all(feature = "alloc", not(feature = "std")))]
use alloc::{collections::BTreeMap, string::String};
#[cfg(feature = "std")]
use std::collections::HashMap;

/// Manages the mapping between command names (e.g., "get_config") and their
/// dynamically assigned message IDs.
#[derive(Debug, Default)]
pub struct CommandRegistry {
    #[cfg(feature = "std")]
    name_to_id: HashMap<String, u8>,
    #[cfg(feature = "std")]
    id_to_name: HashMap<u8, String>,

    #[cfg(all(feature = "alloc", not(feature = "std")))]
    name_to_id: BTreeMap<String, u8>,
    #[cfg(all(feature = "alloc", not(feature = "std")))]
    id_to_name: BTreeMap<u8, String>,
}

impl CommandRegistry {
    /// Creates a new, empty `CommandRegistry`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a command and its ID to the registry.
    ///
    /// # Arguments
    ///
    /// * `name` - The string name of the command.
    /// * `id` - The numeric ID assigned to the command.
    pub fn add(&mut self, name: &str, id: u8) {
        let name_string = String::from(name);
        self.name_to_id.insert(name_string.clone(), id);
        self.id_to_name.insert(id, name_string);
    }

    /// Gets a command ID by its name.
    pub fn get_id(&self, name: &str) -> Option<u8> {
        self.name_to_id.get(name).copied()
    }

    /// Gets a command name by its ID.
    pub fn get_name(&self, id: u8) -> Option<&str> {
        self.id_to_name.get(&id).map(|s| s.as_str())
    }
}

