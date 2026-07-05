#![cfg_attr(not(feature = "std"), no_std)]

use heapless::HistoryBuffer;
use r_klipp_api::FixedPoint;
use embassy_time::{Instant, Duration};

pub trait EmergencyStop {
    fn emergency_stop(&mut self);
}

pub struct ThermalSafetyMonitor {
    temperature_history: HistoryBuffer<FixedPoint, 10>,
    last_check_time: Instant,
}

impl ThermalSafetyMonitor {
    pub fn new() -> Self {
        Self {
            temperature_history: HistoryBuffer::new(),
            last_check_time: Instant::now(),
        }
    }

    pub fn check_runaway(&mut self, current_temp: FixedPoint, pwm_duty_cycle: FixedPoint, estop: &mut impl EmergencyStop) {
        self.temperature_history.write(current_temp);

        if Instant::now() - self.last_check_time >= Duration::from_secs(20) {
            self.last_check_time = Instant::now();

            if pwm_duty_cycle > FixedPoint::from_num(0.5) {
                let oldest_temp = self.temperature_history.oldest().unwrap_or(&current_temp);
                let slope = (current_temp - oldest_temp) / FixedPoint::from_num(20.0);

                if slope < FixedPoint::from_num(0.5) {
                    estop.emergency_stop();
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockEStop {
        tripped: RefCell<bool>,
    }

    impl EmergencyStop for MockEStop {
        fn emergency_stop(&mut self) {
            *self.tripped.borrow_mut() = true;
        }
    }

    #[test]
    fn test_thermal_runaway_detection() {
        let mut monitor = ThermalSafetyMonitor::new();
        let mut estop = MockEStop { tripped: RefCell::new(false) };

        // Simulate a scenario where the temperature rises normally at first
        for i in 0..10 {
            let temp = FixedPoint::from_num(25.0 + (i as f32 * 2.0));
            monitor.check_runaway(temp, FixedPoint::from_num(0.6), &mut estop);
            assert!(!*estop.tripped.borrow());
            // In a real test this would involve advancing mock time
        }

        // Simulate a flattened temperature curve, indicating a problem
        for _ in 0..10 {
            monitor.check_runaway(FixedPoint::from_num(45.0), FixedPoint::from_num(0.6), &mut estop);
        }

        // After 20 seconds of flattened temp, it should trigger
        monitor.check_runaway(FixedPoint::from_num(45.0), FixedPoint::from_num(0.6), &mut estop);
        assert!(*estop.tripped.borrow());
    }
}