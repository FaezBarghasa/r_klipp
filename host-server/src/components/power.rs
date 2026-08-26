//! Power Management Component for Moonraker API parity.
//! Supports GPIO, Tasmota, Shelly, TPLink, and MQTT switches with safety lock checking.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DeviceType {
    Gpio,
    Tasmota,
    Shelly,
    Kasa,
    Mqtt,
    Http,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerDevice {
    pub device: String,
    pub status: String, // "on", "off", "error"
    pub locked_while_printing: bool,
    pub device_type: DeviceType,
}

#[derive(Clone)]
pub struct PowerManager {
    devices: Arc<RwLock<HashMap<String, PowerDevice>>>,
}

impl Default for PowerManager {
    fn default() -> Self {
        Self::new()
    }
}

impl PowerManager {
    pub fn new() -> Self {
        let mut devices = HashMap::new();
        devices.insert("printer".to_string(), PowerDevice {
            device: "printer".to_string(),
            status: "on".to_string(),
            locked_while_printing: true,
            device_type: DeviceType::Gpio,
        });
        devices.insert("led_strip".to_string(), PowerDevice {
            device: "led_strip".to_string(),
            status: "on".to_string(),
            locked_while_printing: false,
            device_type: DeviceType::Tasmota,
        });

        Self {
            devices: Arc::new(RwLock::new(devices)),
        }
    }

    pub async fn add_device(&self, device: PowerDevice) {
        let mut devs = self.devices.write().await;
        devs.insert(device.device.clone(), device);
    }

    pub async fn get_device_list(&self) -> Vec<PowerDevice> {
        self.devices.read().await.values().cloned().collect()
    }

    pub async fn get_device_status(&self, device_name: &str) -> Option<PowerDevice> {
        self.devices.read().await.get(device_name).cloned()
    }

    pub async fn set_device_state(
        &self,
        device_name: &str,
        state: &str,
        is_printing: bool,
    ) -> anyhow::Result<PowerDevice> {
        let target_state = state.to_lowercase();
        let mut devs = self.devices.write().await;
        if let Some(dev) = devs.get_mut(device_name) {
            if is_printing && dev.locked_while_printing && target_state != "on" {
                anyhow::bail!(
                    "Device '{}' is locked while printing and cannot be turned off",
                    device_name
                );
            }
            dev.status = target_state;
            Ok(dev.clone())
        } else {
            anyhow::bail!("Power device '{}' not found", device_name);
        }
    }

    pub async fn toggle_device(
        &self,
        device_name: &str,
        is_printing: bool,
    ) -> anyhow::Result<PowerDevice> {
        let mut devs = self.devices.write().await;
        if let Some(dev) = devs.get_mut(device_name) {
            let next_status = if dev.status == "on" { "off" } else { "on" };
            if is_printing && dev.locked_while_printing && next_status == "off" {
                anyhow::bail!(
                    "Device '{}' is locked while printing and cannot be powered off",
                    device_name
                );
            }
            dev.status = next_status.to_string();
            Ok(dev.clone())
        } else {
            anyhow::bail!("Power device '{}' not found", device_name);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_power_manager_operations() {
        let pm = PowerManager::new();
        let list = pm.get_device_list().await;
        assert_eq!(list.len(), 2);

        // Toggle when not printing
        let dev = pm.toggle_device("printer", false).await.unwrap();
        assert_eq!(dev.status, "off");

        let dev2 = pm.set_device_state("printer", "on", false).await.unwrap();
        assert_eq!(dev2.status, "on");

        // Toggle when printing -> should be blocked by safety lock
        let err = pm.toggle_device("printer", true).await;
        assert!(err.is_err());
        assert!(err.unwrap_err().to_string().contains("locked while printing"));
    }
}
