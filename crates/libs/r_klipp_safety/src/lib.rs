#![cfg_attr(not(feature = "std"), no_std)]

use heapless::HistoryBuffer;

pub struct ThermalSafetyMonitor {
    temperature_history: HistoryBuffer<f32, 10>,
    last_check_time: embassy_time::Instant,
}

impl ThermalSafetyMonitor {
    pub fn new() -> Self {
        Self {
            temperature_history: HistoryBuffer::new(),
            last_check_time: embassy_time::Instant::now(),
        }
    }

    pub fn check_runaway(&mut self, current_temp: f32, pwm_duty_cycle: f32) -> bool {
        self.temperature_history.write(current_temp);

        if embassy_time::Instant::now() - self.last_check_time >= embassy_time::Duration::from_secs(20) {
            self.last_check_time = embassy_time::Instant::now();

            if pwm_duty_cycle > 0.5 {
                let oldest_temp = self.temperature_history.oldest().unwrap_or(&current_temp);
                let slope = (current_temp - oldest_temp) / 20.0; // 20 seconds

                if slope < 0.5 {
                    return true; // Trigger E-Stop
                }
            }
        }
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use embassy_time::{Duration, Instant};

    #[test]
    fn test_thermal_runaway_detection() {
        let mut monitor = ThermalSafetyMonitor::new();
        let start_time = Instant::now();

        // Simulate a scenario where the temperature rises normally at first
        for i in 0..10 {
            let temp = 25.0 + (i as f32 * 2.0);
            assert!(!monitor.check_runaway(temp, 0.6));
            embassy_time::block_for(Duration::from_secs(2));
        }

        // Simulate a flattened temperature curve, indicating a problem
        for _ in 0..10 {
            assert!(!monitor.check_runaway(45.0, 0.6));
            embassy_time::block_for(Duration::from_secs(2));
        }

        // After 20 seconds of flattened temp, it should trigger
        assert!(monitor.check_runaway(45.0, 0.6));
    }
}