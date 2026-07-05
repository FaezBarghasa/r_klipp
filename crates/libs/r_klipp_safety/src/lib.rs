#![no_std]

pub struct BufferStarvationWatchdog {
    deceleration_triggered: bool,
}

impl BufferStarvationWatchdog {
    pub fn new() -> Self {
        Self {
            deceleration_triggered: false,
        }
    }

    pub fn check(&mut self, buffer_fill_percent: u8) -> WatchdogAction {
        if buffer_fill_percent < 5 && !self.deceleration_triggered {
            self.deceleration_triggered = true;
            WatchdogAction::SafeDecelerate
        } else if buffer_fill_percent == 0 {
            WatchdogAction::JerkLimitedPause
        } else {
            if buffer_fill_percent > 10 { // Hysteresis
                self.deceleration_triggered = false;
            }
            WatchdogAction::None
        }
    }
}

#[derive(Debug, PartialEq)]
pub enum WatchdogAction {
    None,
    SafeDecelerate,
    JerkLimitedPause,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_buffer_starvation_watchdog() {
        let mut watchdog = BufferStarvationWatchdog::new();
        assert_eq!(watchdog.check(50), WatchdogAction::None);
        assert_eq!(watchdog.check(6), WatchdogAction::None);
        assert_eq!(watchdog.check(4), WatchdogAction::SafeDecelerate);
        assert_eq!(watchdog.check(3), WatchdogAction::None); // Already triggered
        assert_eq!(watchdog.check(0), WatchdogAction::JerkLimitedPause);
        assert_eq!(watchdog.check(11), WatchdogAction::None);
        assert_eq!(watchdog.check(4), WatchdogAction::SafeDecelerate); // Can trigger again
    }
}
