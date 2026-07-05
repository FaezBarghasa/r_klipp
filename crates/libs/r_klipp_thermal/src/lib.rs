#![no_std]

use heapless::Vec;

pub struct ThermalSafetyMonitor {
    temp_history: Vec<f32, 20>,
    last_temp: f32,
}

impl ThermalSafetyMonitor {
    pub fn new() -> Self {
        Self {
            temp_history: Vec::new(),
            last_temp: 0.0,
        }
    }

    // Using Steinhart-Hart equation for thermistor reading
    pub fn adc_to_temp(adc_val: u16) -> f32 {
        // Dummy implementation
        adc_val as f32 / 10.0
    }

    pub fn check_runaway(&mut self, current_temp: f32, pwm_duty_cycle: f32) -> bool {
        if self.temp_history.is_full() {
            let slope = (current_temp - self.temp_history[0]) / 20.0;
            if pwm_duty_cycle > 0.5 && slope < 0.5 {
                return true; // Potential runaway
            }
            self.temp_history.remove(0);
        }
        self.temp_history.push(current_temp).unwrap();
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thermal_runaway_detection() {
        let mut monitor = ThermalSafetyMonitor::new();
        for _ in 0..20 {
            assert!(!monitor.check_runaway(25.0, 0.6));
        }
        // After 20s of no temp change with high PWM, it should trigger
        assert!(monitor.check_runaway(25.0, 0.6));
    }
}
