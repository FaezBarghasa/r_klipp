//! StallGuard and Motor Current Surveillance for Paste / Solder Dispensing.
//!
//! Monitors driver load metrics (e.g. TMC2240 SG_RESULT / phase current). If back-pressure
//! or syringe needle blockage spikes current beyond calibrated limits, instantly flags
//! an axis stall to prevent mechanical syringe destruction.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StallGuardSurveillance {
    pub current_limit_ma: u16,
    pub stall_threshold_sg: i32, // TMC StallGuard threshold (-64 to +63)
    pub is_stalled: bool,
    pub consecutive_fault_samples: u8,
    pub fault_tolerance_samples: u8,
}

impl StallGuardSurveillance {
    pub fn new(current_limit_ma: u16, stall_threshold_sg: i32) -> Self {
        Self {
            current_limit_ma,
            stall_threshold_sg,
            is_stalled: false,
            consecutive_fault_samples: 0,
            fault_tolerance_samples: 3, // Require 3 consecutive trip samples to filter noise
        }
    }

    /// Evaluates live driver metrics from TMC register read.
    /// Returns true if an unrecoverable stall / syringe over-pressure event is confirmed.
    pub fn update_driver_feedback(&mut self, current_ma: u16, sg_result: i32) -> bool {
        let over_current = current_ma > self.current_limit_ma;
        let sg_stall = sg_result <= self.stall_threshold_sg;

        if over_current || sg_stall {
            self.consecutive_fault_samples = self.consecutive_fault_samples.saturating_add(1);
            if self.consecutive_fault_samples >= self.fault_tolerance_samples {
                self.is_stalled = true;
            }
        } else {
            self.consecutive_fault_samples = 0;
            self.is_stalled = false;
        }

        self.is_stalled
    }

    pub fn reset_fault(&mut self) {
        self.is_stalled = false;
        self.consecutive_fault_samples = 0;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_stall_detection_noise_filtering_and_trip() {
        let mut sg = StallGuardSurveillance::new(1500, 10);

        // Single noise spike -> should NOT trip
        assert!(!sg.update_driver_feedback(1800, 5));
        assert!(!sg.is_stalled);

        // Second spike
        assert!(!sg.update_driver_feedback(1800, 5));

        // Third consecutive spike -> confirms stall
        assert!(sg.update_driver_feedback(1800, 5));
        assert!(sg.is_stalled);

        // Reset
        sg.reset_fault();
        assert!(!sg.is_stalled);
    }
}
