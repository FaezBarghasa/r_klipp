/// Configuration parameters for the thermal runaway watchdog
#[derive(Clone, Copy, Debug)]
pub struct RunawayWatchdogConfig {
    pub max_temp_c: f32,
    pub min_temp_c: f32,
    pub max_heating_rate_c_per_s: f32,
    pub runaway_timeout_ms: u32,
    pub hysteresis_temp_c: f32,
}

impl Default for RunawayWatchdogConfig {
    fn default() -> Self {
        Self {
            max_temp_c: 300.0,
            min_temp_c: -10.0,
            max_heating_rate_c_per_s: 15.0,
            runaway_timeout_ms: 10_000, // 10 seconds
            hysteresis_temp_c: 5.0,
        }
    }
}

/// Thermal Runaway Watchdog & Hardware Fail-safe Supervisor
pub struct RunawayWatchdog {
    pub config: RunawayWatchdogConfig,
    pub last_temp_c: f32,
    pub last_timestamp_ms: u32,
    pub runaway_timer_start_ms: Option<u32>,
    pub e_stop_triggered: bool,
}

impl RunawayWatchdog {
    pub fn new(config: RunawayWatchdogConfig, initial_temp: f32, initial_timestamp_ms: u32) -> Self {
        Self {
            config,
            last_temp_c: initial_temp,
            last_timestamp_ms: initial_timestamp_ms,
            runaway_timer_start_ms: None,
            e_stop_triggered: false,
        }
    }

    pub fn is_e_stop_triggered(&self) -> bool {
        self.e_stop_triggered
    }

    /// Evaluates thermal boundaries and runaway conditions. Returns `true` if E-Stop is triggered.
    pub fn check_temperature(&mut self, current_temp_c: f32, current_timestamp_ms: u32, heater_active: bool) -> bool {
        if self.e_stop_triggered {
            return true;
        }

        // 1. Min/Max absolute limits
        if current_temp_c > self.config.max_temp_c || current_temp_c < self.config.min_temp_c {
            self.e_stop_triggered = true;
            return true;
        }

        let dt_s = (current_timestamp_ms.wrapping_sub(self.last_timestamp_ms) as f32) / 1000.0;
        if dt_s > 0.05 {
            let rate = (current_temp_c - self.last_temp_c) / dt_s;
            // Negative heating rate when heater is at 100% indicates detached sensor
            if heater_active && rate < -2.0 {
                if let Some(start) = self.runaway_timer_start_ms {
                    if current_timestamp_ms.wrapping_sub(start) > self.config.runaway_timeout_ms {
                        self.e_stop_triggered = true;
                        return true;
                    }
                } else {
                    self.runaway_timer_start_ms = Some(current_timestamp_ms);
                }
            } else {
                self.runaway_timer_start_ms = None;
            }

            self.last_temp_c = current_temp_c;
            self.last_timestamp_ms = current_timestamp_ms;
        }

        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalState {
    Ok,
    Warning,
    Shutdown,
}

#[derive(Debug, Clone, Copy)]
pub struct SafetyLimits {
    pub max_temp: f64,
    pub min_heat_gain_temp: f64,
    pub min_heat_gain_time_s: f64,
    pub max_deviation: f64,
}

pub struct HeaterSafety {
    limits: SafetyLimits,
    start_time: Option<f64>,
    start_temp: f64,
    heater_on: bool,
}

impl HeaterSafety {
    pub fn new(limits: SafetyLimits) -> Self {
        Self {
            limits,
            start_time: None,
            start_temp: 0.0,
            heater_on: false,
        }
    }

    pub fn update(&mut self, time_s: f64, current_temp: f64, _target_temp: f64, heater_on: bool) -> ThermalState {
        if current_temp > self.limits.max_temp {
            return ThermalState::Shutdown;
        }

        if heater_on {
            if !self.heater_on || self.start_time.is_none() {
                self.heater_on = true;
                self.start_time = Some(time_s);
                self.start_temp = current_temp;
            } else if let Some(t0) = self.start_time {
                let dt = time_s - t0;
                if dt >= self.limits.min_heat_gain_time_s {
                    let dtemp = current_temp - self.start_temp;
                    if dtemp < self.limits.min_heat_gain_temp {
                        return ThermalState::Shutdown;
                    }
                }
            }
        } else {
            self.heater_on = false;
            self.start_time = None;
        }

        ThermalState::Ok
    }
}