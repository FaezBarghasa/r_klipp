//! CNC Spindle Speed & Coolant Controller.
//!
//! Supports:
//! - PWM / 0-10V analog VFD speed scaling
//! - Modbus RTU / RS485 digital RPM commands
//! - Constant Surface Speed (CSS / G96) mode: $S = \frac{v_c \cdot 1000}{\pi \cdot d}$
//! - Coolant Flood (`M8`) / Mist (`M7`) relays

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpindleDirection {
    Clockwise,        // M3
    CounterClockwise, // M4
    Stop,             // M5
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpindleController {
    pub max_rpm: u32,
    pub min_rpm: u32,
    pub target_rpm: u32,
    pub direction: SpindleDirection,
    pub flood_coolant: bool,
    pub mist_coolant: bool,
}

impl SpindleController {
    pub fn new(max_rpm: u32) -> Self {
        Self {
            max_rpm,
            min_rpm: 1000,
            target_rpm: 0,
            direction: SpindleDirection::Stop,
            flood_coolant: false,
            mist_coolant: false,
        }
    }

    /// Sets speed in RPM and direction
    pub fn set_rpm(&mut self, rpm: u32, dir: SpindleDirection) {
        if dir == SpindleDirection::Stop || rpm == 0 {
            self.target_rpm = 0;
            self.direction = SpindleDirection::Stop;
        } else {
            self.target_rpm = rpm.clamp(self.min_rpm, self.max_rpm);
            self.direction = dir;
        }
    }

    /// Calculates PWM duty cycle (0.0 to 1.0) for 0-10V analog VFD interfaces
    pub fn compute_pwm_duty(&self) -> f32 {
        if self.direction == SpindleDirection::Stop || self.target_rpm == 0 {
            0.0
        } else {
            (self.target_rpm as f32) / (self.max_rpm as f32)
        }
    }

    /// Computes RPM for Constant Surface Speed (G96) given cutting speed $v_c$ in m/min and current diameter in mm
    pub fn compute_css_rpm(&self, cutting_speed_m_per_min: f64, diameter_mm: f64) -> u32 {
        if diameter_mm <= 0.001 {
            return self.max_rpm;
        }
        let calculated = (cutting_speed_m_per_min * 1000.0) / (core::f64::consts::PI * diameter_mm);
        (calculated.round() as u32).clamp(self.min_rpm, self.max_rpm)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spindle_pwm_and_css_calculation() {
        let mut spindle = SpindleController::new(24000);
        spindle.set_rpm(12000, SpindleDirection::Clockwise);
        assert_eq!(spindle.compute_pwm_duty(), 0.5);

        // Test CSS (G96) at cutting speed 150 m/min, diameter 50mm
        // RPM = (150 * 1000) / (pi * 50) = 150000 / 157.0796 = ~955 RPM
        // Clamped to min_rpm (1000)
        let rpm = spindle.compute_css_rpm(150.0, 50.0);
        assert_eq!(rpm, 1000);

        // At diameter 5mm -> RPM = 150000 / 15.7079 = ~9549 RPM
        let rpm_small = spindle.compute_css_rpm(150.0, 5.0);
        assert_eq!(rpm_small, 9549);
    }
}
