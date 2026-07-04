#![no_std]
use libm;
use crate::tuner_trait::{AutoTuner, TunerState};

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TuneMode {
    Heating,
    Cooling,
}

pub struct UpRelayAutotuner {
    setpoint: f32,
    hysteresis: f32,
    output: f32,
    mode: TuneMode,
    state: TunerState,
    runtime: f32,
    peak_count: u32,
    peaks: [f32; 10],
    peak_times: [f32; 10],
    output_state: bool,
    last_crossing_time: f32,
    kp: f32,
    ki: f32,
    kd: f32,
}

impl UpRelayAutotuner {
    pub fn new(setpoint: f32, hysteresis: f32, output: f32, mode: TuneMode) -> Self {
        Self {
            setpoint,
            hysteresis,
            output,
            mode,
            state: TunerState::Idle,
            runtime: 0.0,
            peak_count: 0,
            peaks: [0.0; 10],
            peak_times: [0.0; 10],
            output_state: false,
            last_crossing_time: 0.0,
            kp: 0.0,
            ki: 0.0,
            kd: 0.0,
        }
    }

    pub fn get_tunings(&self) -> Option<(f32, f32, f32)> {
        if self.state == TunerState::Complete {
            Some((self.kp, self.ki, self.kd))
        } else {
            None
        }
    }
}

impl AutoTuner for UpRelayAutotuner {
    fn update(&mut self, _setpoint: f32, measurement: f32, time: f32) -> (TunerState, f32) {
        self.runtime = time;

        if self.state == TunerState::Idle {
            self.state = TunerState::Running;
            self.last_crossing_time = time;
        }

        if self.state == TunerState::Running {
            let (upper_bound, lower_bound) = match self.mode {
                TuneMode::Heating => (self.setpoint + self.hysteresis, self.setpoint - self.hysteresis),
                TuneMode::Cooling => (self.setpoint + self.hysteresis, self.setpoint - self.hysteresis),
            };

            if measurement > upper_bound {
                self.output_state = false;
                if self.peak_count > 0 && self.peak_times[self.peak_count as usize - 1] < self.last_crossing_time {
                    self.peaks[self.peak_count as usize] = measurement;
                    self.peak_times[self.peak_count as usize] = time;
                    self.peak_count += 1;
                }
            } else if measurement < lower_bound {
                self.output_state = true;
                if self.peak_count > 0 && self.peak_times[self.peak_count as usize - 1] > self.last_crossing_time {
                    self.peaks[self.peak_count as usize] = measurement;
                    self.peak_times[self.peak_count as usize] = time;
                    self.peak_count += 1;
                }
            }

            if self.peak_count >= 10 {
                let (amplitude, period) = self.calculate_amplitude_and_period();
                let (kp, ki, kd) = self.ziegler_nichols(amplitude, period);
                self.kp = kp;
                self.ki = ki;
                self.kd = kd;
                self.state = TunerState::Complete;
            }
        }

        let output_signal = if self.output_state { self.output } else { 0.0 };
        (self.state, output_signal)
    }

    fn get_gains(&self) -> Option<(f32, f32, f32)> {
        self.get_tunings()
    }

    fn reset(&mut self) {
        self.state = TunerState::Idle;
        self.runtime = 0.0;
        self.peak_count = 0;
        self.peaks = [0.0; 10];
        self.peak_times = [0.0; 10];
        self.output_state = false;
        self.last_crossing_time = 0.0;
        self.kp = 0.0;
        self.ki = 0.0;
        self.kd = 0.0;
    }
}

impl UpRelayAutotuner {
    fn calculate_amplitude_and_period(&self) -> (f32, f32) {
        let mut max_peak = 0.0;
        let mut min_peak = 0.0;
        let mut period_sum = 0.0;

        for i in 1..self.peak_count as usize {
            if self.peaks[i] > self.setpoint {
                max_peak += self.peaks[i];
            } else {
                min_peak += self.peaks[i];
            }
            period_sum += self.peak_times[i] - self.peak_times[i - 1];
        }

        let max_peak_avg = max_peak / (self.peak_count / 2) as f32;
        let min_peak_avg = min_peak / (self.peak_count / 2) as f32;
        let amplitude = (max_peak_avg - min_peak_avg) / 2.0;
        let period = period_sum / (self.peak_count - 1) as f32;

        (amplitude, period)
    }

    fn ziegler_nichols(&self, amplitude: f32, period: f32) -> (f32, f32, f32) {
        let ku = 4.0 * self.output / (libm::acos(-1.0) * amplitude);
        let tu = period;

        let kp = 0.6 * ku;
        let ki = 1.2 * ku / tu;
        let kd = 0.075 * ku * tu;

        (kp, ki, kd)
    }
}
