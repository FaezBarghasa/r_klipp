use anyhow::{anyhow, Result};
use dashmap::DashMap;
use std::any::Any;
use std::sync::Arc;
use tokio::sync::broadcast;

use super::component::Component;
use super::datastore::DataStore;
use crate::db::Database;

/// Thread-safe application context passed to all components and handlers.
#[derive(Clone)]
pub struct Context {
    components: Arc<DashMap<String, Arc<dyn Component>>>,
    pub datastore: DataStore,
    pub db: Arc<Database>,
    pub event_bus: broadcast::Sender<serde_json::Value>,
}

impl Context {
    pub fn new(db: Arc<Database>, event_bus: broadcast::Sender<serde_json::Value>) -> Self {
        Self {
            components: Arc::new(DashMap::new()),
            datastore: DataStore::new(),
            db,
            event_bus,
        }
    }

    /// Register a component in the global registry.
    pub fn register_component(&self, component: Arc<dyn Component>) {
        self.components
            .insert(component.name().to_string(), component);
    }

    /// Lookup a component by name.
    pub fn get_component(&self, name: &str) -> Option<Arc<dyn Component>> {
        self.components.get(name).map(|c| c.clone())
    }

    /// Lookup a component by type.
    pub fn lookup_component<T: 'static>(&self, name: &str) -> Result<Arc<T>> {
        let comp = self
            .get_component(name)
            .ok_or_else(|| anyhow!("Component '{}' not found", name))?;

        // Downcast helper: requires concrete struct to match
        // Note: As an abstraction, downcasting is done through Any
        let any_ref = comp.as_any();
        if any_ref.is::<T>() {
            // Note: Since comp is Arc<dyn Component>, downcasting Arc requires unsafe or downcast_rs.
            // For general lookup, components can provide typed accessors or traits.
        }
        Err(anyhow!("Downcasting component '{}' requires concrete trait method", name))
    }
}
