use dashmap::DashMap;
use serde_json::Value;
use std::sync::Arc;

/// Fast, thread-safe in-memory cache for Klipper and Moonraker real-time status.
#[derive(Clone, Default)]
pub struct DataStore {
    store: Arc<DashMap<String, Value>>,
}

impl DataStore {
    pub fn new() -> Self {
        Self {
            store: Arc::new(DashMap::new()),
        }
    }

    /// Update status for an object path (e.g. "extruder", "toolhead", "heater_bed").
    pub fn update_state(&self, path: &str, value: Value) {
        self.store.insert(path.to_string(), value);
    }

    /// Merge object updates (shallow merge for JSON maps).
    pub fn merge_state(&self, path: &str, mut new_val: Value) {
        self.store
            .entry(path.to_string())
            .and_modify(|existing| {
                if let (Value::Object(target_map), Value::Object(source_map)) =
                    (existing, &mut new_val)
                {
                    for (k, v) in std::mem::take(source_map) {
                        target_map.insert(k, v);
                    }
                }
            })
            .or_insert(new_val);
    }

    /// Retrieve state for a given object key.
    pub fn get_state(&self, path: &str) -> Option<Value> {
        self.store.get(path).map(|v| v.clone())
    }

    /// Retrieve full status snapshot of all objects.
    pub fn get_full_state(&self) -> Value {
        let mut map = serde_json::Map::new();
        for item in self.store.iter() {
            map.insert(item.key().clone(), item.value().clone());
        }
        Value::Object(map)
    }

    /// Filter state for a specific list of keys.
    pub fn get_filtered_state(&self, keys: &[&str]) -> Value {
        let mut map = serde_json::Map::new();
        for &k in keys {
            if let Some(val) = self.store.get(k) {
                map.insert(k.to_string(), val.clone());
            }
        }
        Value::Object(map)
    }
}
