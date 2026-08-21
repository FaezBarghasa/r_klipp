//! Dynamic mapping of command names to IDs and versioning helpers.

use alloc::collections::BTreeMap;
use alloc::string::String;

/// Manages the mapping between command names (e.g., "get_config") and their
/// dynamically assigned message IDs.
#[derive(Debug, Default)]
pub struct CommandRegistry {
    name_to_id: BTreeMap<String, u8>,
    id_to_name: BTreeMap<u8, String>,
}

impl CommandRegistry {
    /// Creates a new, empty `CommandRegistry`.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a command and its ID to the registry.
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
