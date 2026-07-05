use embassy_time::Instant;
use micromath::F32Ext;

pub trait Clock {
    fn get_freq(&self) -> u32;
    fn set_freq(&mut self, freq: u32);
}

pub struct Dpll {
    kp: f32,
    ki: f32,
    integral: f32,
    base_freq: u32,
}

impl Dpll {
    pub fn new(base_freq: u32, kp: f32, ki: f32) -> Self {
        Self {
            kp,
            ki,
            integral: 0.0,
            base_freq,
        }
    }

    pub fn sync_clock(&mut self, clock: &mut impl Clock, host_timestamp: u64) {
        let mcu_timestamp = Instant::now().as_micros();
        let phase_error = (host_timestamp as i64 - mcu_timestamp as i64) as f32;

        self.integral += phase_error * self.ki;
        // Clamp integral to prevent windup
        self.integral = self.integral.clamp(-1000.0, 1000.0);

        let correction = phase_error * self.kp + self.integral;
        let new_freq = self.base_freq as f32 + correction;

        clock.set_freq(new_freq as u32);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::RefCell;

    struct MockClock {
        freq: RefCell<u32>,
    }

    impl Clock for MockClock {
        fn get_freq(&self) -> u32 {
            *self.freq.borrow()
        }
        fn set_freq(&mut self, freq: u32) {
            *self.freq.borrow_mut() = freq;
        }
    }

    #[test]
    fn test_dpll_correction() {
        let mut clock = MockClock { freq: RefCell::new(1_000_000) };
        let mut dpll = Dpll::new(1_000_000, 0.1, 0.01);

        // Simulate host time being ahead
        let host_timestamp = Instant::now().as_micros() + 100;
        dpll.sync_clock(&mut clock, host_timestamp);
        assert!(clock.get_freq() > 1_000_000);

        // Simulate host time being behind
        let host_timestamp = Instant::now().as_micros() - 100;
        dpll.sync_clock(&mut clock, host_timestamp);
        assert!(clock.get_freq() < 1_000_000);
    }
}