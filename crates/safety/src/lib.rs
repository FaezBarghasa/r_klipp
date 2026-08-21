#![no_std]

pub mod probe_interlock;
pub use probe_interlock::{ProbeSupervisor, ProbeState, EnclosureInterlock};

use core::sync::atomic::{AtomicBool, AtomicU32, AtomicU8, Ordering};

pub const ABSOLUTE_MAX_TEMPERATURE: f32 = 320.0;

/// Reasons for an emergency stop.
#[repr(u8)]
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum EStopReason {
    None = 0,
    ThermalRunaway = 1,
    TemperatureOutOfBounds = 2,
    MotorFault = 3,
    HostCommand = 4,
    WatchdogTimeout = 5,
    Other = 255,
}

impl From<u8> for EStopReason {
    fn from(value: u8) -> Self {
        match value {
            1 => EStopReason::ThermalRunaway,
            2 => EStopReason::TemperatureOutOfBounds,
            3 => EStopReason::MotorFault,
            4 => EStopReason::HostCommand,
            5 => EStopReason::WatchdogTimeout,
            _ => EStopReason::Other,
        }
    }
}

/// Hardware Independent Watchdog (IWDG) Controller.
/// Must be fed periodically by the main event loop. If execution blocks for longer than `timeout_ms`,
/// the hardware watchdog triggers an emergency reset/shutdown.
pub struct IndependentWatchdog {
    pub timeout_ms: u32,
    last_feed_tick: AtomicU32,
    armed: AtomicBool,
}

impl IndependentWatchdog {
    pub const fn new(timeout_ms: u32) -> Self {
        Self {
            timeout_ms,
            last_feed_tick: AtomicU32::new(0),
            armed: AtomicBool::new(false),
        }
    }

    pub fn arm(&self, current_tick_ms: u32) {
        self.last_feed_tick.store(current_tick_ms, Ordering::Release);
        self.armed.store(true, Ordering::Release);
    }

    pub fn feed(&self, current_tick_ms: u32) {
        self.last_feed_tick.store(current_tick_ms, Ordering::Release);
    }

    pub fn check(&self, current_tick_ms: u32) -> Result<(), EStopReason> {
        if !self.armed.load(Ordering::Acquire) {
            return Ok(());
        }

        let last = self.last_feed_tick.load(Ordering::Acquire);
        let elapsed = current_tick_ms.saturating_sub(last);

        if elapsed > self.timeout_ms {
            trigger_e_stop(EStopReason::WatchdogTimeout);
            Err(EStopReason::WatchdogTimeout)
        } else {
            Ok(())
        }
    }
}

/// Global E-Stop state accessible across the MCU.
pub struct EStopState {
    is_active: AtomicBool,
    reason: AtomicU8,
}

impl EStopState {
    pub const fn new() -> Self {
        Self {
            is_active: AtomicBool::new(false),
            reason: AtomicU8::new(EStopReason::None as u8),
        }
    }

    pub fn trigger(&self, reason: EStopReason) {
        self.reason.store(reason as u8, Ordering::Release);
        self.is_active.store(true, Ordering::Release);
    }

    pub fn clear(&self) {
        self.is_active.store(false, Ordering::Release);
        self.reason.store(EStopReason::None as u8, Ordering::Release);
    }

    pub fn is_active(&self) -> bool {
        self.is_active.load(Ordering::Acquire)
    }

    pub fn get_reason(&self) -> EStopReason {
        self.reason.load(Ordering::Acquire).into()
    }
}

static GLOBAL_ESTOP_STATE: EStopState = EStopState::new();

pub fn trigger_e_stop(reason: EStopReason) {
    GLOBAL_ESTOP_STATE.trigger(reason);
}

pub fn clear_e_stop() {
    GLOBAL_ESTOP_STATE.clear();
}

pub fn is_e_stop_active() -> bool {
    GLOBAL_ESTOP_STATE.is_active()
}

pub fn get_e_stop_reason() -> EStopReason {
    GLOBAL_ESTOP_STATE.get_reason()
}

#[derive(Debug, Clone, Copy)]
pub struct RunawayWatchdogConfig {
    pub min_temp_threshold: f32,
    pub max_temp_threshold: f32,
    pub max_temp_change_rate: f32,
    pub min_time_for_rate_ms: u32,
}

pub struct RunawayWatchdog {
    config: RunawayWatchdogConfig,
    last_temp: f32,
    last_timestamp_ms: u32,
}

impl RunawayWatchdog {
    pub fn new(config: RunawayWatchdogConfig, initial_temp: f32, initial_timestamp_ms: u32) -> Self {
        Self {
            config,
            last_temp: initial_temp,
            last_timestamp_ms: initial_timestamp_ms,
        }
    }

    pub fn check_temperature(&mut self, current_temp: f32, current_timestamp_ms: u32, heater_active: bool) -> bool {
        if is_e_stop_active() {
            return true;
        }

        if current_temp < self.config.min_temp_threshold || current_temp > self.config.max_temp_threshold {
            trigger_e_stop(EStopReason::TemperatureOutOfBounds);
            return true;
        }

        let dt_ms = current_timestamp_ms.saturating_sub(self.last_timestamp_ms);

        if heater_active && dt_ms >= self.config.min_time_for_rate_ms {
            let temp_change = current_temp - self.last_temp;
            let time_change_s = dt_ms as f32 / 1000.0;

            if time_change_s > 0.0 {
                let temp_change_rate = temp_change / time_change_s;

                if temp_change_rate > self.config.max_temp_change_rate {
                    trigger_e_stop(EStopReason::ThermalRunaway);
                    return true;
                }
            }
        }

        self.last_temp = current_temp;
        self.last_timestamp_ms = current_timestamp_ms;
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_iwdg_watchdog_timeout() {
        let iwdg = IndependentWatchdog::new(500); // 500ms timeout
        iwdg.arm(1000);

        assert!(iwdg.check(1200).is_ok());
        iwdg.feed(1200);

        assert!(iwdg.check(1500).is_ok());

        // Timeout triggers
        assert_eq!(iwdg.check(1800), Err(EStopReason::WatchdogTimeout));
        assert!(is_e_stop_active());
        assert_eq!(get_e_stop_reason(), EStopReason::WatchdogTimeout);

        clear_e_stop();
    }
}
