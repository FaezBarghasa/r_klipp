//! Time-series ring buffer for storing temperature and sensor history.
//! 1-to-1 replacement of Moonraker's `data_store.py`.

use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorDataPoint {
    pub time: f64,
    pub temperature: f32,
    pub target: f32,
    pub power: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorHistory {
    pub name: String,
    pub temperatures: Vec<f32>,
    pub targets: Vec<f32>,
    pub powers: Vec<f32>,
    pub times: Vec<f64>,
}

#[derive(Clone)]
pub struct DataStore {
    max_history_seconds: f64,
    sensors: Arc<RwLock<HashMap<String, VecDeque<SensorDataPoint>>>>,
}

impl DataStore {
    pub fn new(max_history_seconds: f64) -> Self {
        Self {
            max_history_seconds,
            sensors: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_point(&self, sensor_name: &str, time: f64, temperature: f32, target: f32, power: f32) {
        let mut sensors = self.sensors.write().await;
        let deque = sensors.entry(sensor_name.to_string()).or_insert_with(VecDeque::new);

        deque.push_back(SensorDataPoint {
            time,
            temperature,
            target,
            power,
        });

        let cutoff = time - self.max_history_seconds;
        while let Some(front) = deque.front() {
            if front.time < cutoff {
                deque.pop_front();
            } else {
                break;
            }
        }
    }

    pub async fn get_sensor_history(&self, sensor_name: &str) -> Option<SensorHistory> {
        let sensors = self.sensors.read().await;
        let deque = sensors.get(sensor_name)?;

        let mut temperatures = Vec::with_capacity(deque.len());
        let mut targets = Vec::with_capacity(deque.len());
        let mut powers = Vec::with_capacity(deque.len());
        let mut times = Vec::with_capacity(deque.len());

        for pt in deque {
            temperatures.push(pt.temperature);
            targets.push(pt.target);
            powers.push(pt.power);
            times.push(pt.time);
        }

        Some(SensorHistory {
            name: sensor_name.to_string(),
            temperatures,
            targets,
            powers,
            times,
        })
    }

    pub async fn get_all_sensors(&self) -> HashMap<String, SensorHistory> {
        let sensors = self.sensors.read().await;
        let mut res = HashMap::new();

        for (name, deque) in sensors.iter() {
            let mut temperatures = Vec::with_capacity(deque.len());
            let mut targets = Vec::with_capacity(deque.len());
            let mut powers = Vec::with_capacity(deque.len());
            let mut times = Vec::with_capacity(deque.len());

            for pt in deque {
                temperatures.push(pt.temperature);
                targets.push(pt.target);
                powers.push(pt.power);
                times.push(pt.time);
            }

            res.insert(name.clone(), SensorHistory {
                name: name.clone(),
                temperatures,
                targets,
                powers,
                times,
            });
        }

        res
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_data_store_ring_buffer() {
        let store = DataStore::new(10.0);
        store.add_point("extruder", 100.0, 200.0, 205.0, 0.5).await;
        store.add_point("extruder", 105.0, 203.0, 205.0, 0.4).await;
        store.add_point("extruder", 112.0, 205.0, 205.0, 0.2).await;

        let hist = store.get_sensor_history("extruder").await.unwrap();
        // 100.0 should be evicted since cutoff is 112.0 - 10.0 = 102.0
        assert_eq!(hist.times.len(), 2);
        assert_eq!(hist.times[0], 105.0);
        assert_eq!(hist.times[1], 112.0);
    }
}
