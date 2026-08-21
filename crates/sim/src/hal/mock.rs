//! Mock Hardware Abstraction Layer (HAL) for deterministic host-side simulation.

use std::sync::{Arc, Mutex};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MockStepEvent {
    pub tick_timestamp: u64,
    pub pin_index: u8,
    pub high: bool,
    pub dir_high: bool,
}

#[derive(Debug, Clone, Default)]
pub struct MockHalState {
    pub current_tick: u64,
    pub step_log: Vec<MockStepEvent>,
    pub pin_states: [bool; 32],
    pub pwm_duties: [f32; 8],
    pub adc_readings: [f32; 8],
    pub endstops_triggered: [bool; 6],
}

#[derive(Clone, Default)]
pub struct MockHal {
    state: Arc<Mutex<MockHalState>>,
}

impl MockHal {
    pub fn new() -> Self {
        Self {
            state: Arc::new(Mutex::new(MockHalState::default())),
        }
    }

    pub fn reset(&self) {
        let mut state = self.state.lock().unwrap();
        *state = MockHalState::default();
    }

    pub fn advance_ticks(&self, ticks: u64) {
        let mut state = self.state.lock().unwrap();
        state.current_tick += ticks;
    }

    pub fn current_tick(&self) -> u64 {
        self.state.lock().unwrap().current_tick
    }

    pub fn step_pulse(&self, pin_index: u8, dir_high: bool) {
        let mut state = self.state.lock().unwrap();
        let current_tick = state.current_tick;
        state.step_log.push(MockStepEvent {
            tick_timestamp: current_tick,
            pin_index,
            high: true,
            dir_high,
        });
        state.step_log.push(MockStepEvent {
            tick_timestamp: current_tick + 10,
            pin_index,
            high: false,
            dir_high,
        });
    }

    pub fn set_pwm_duty(&self, channel: usize, duty: f32) {
        let mut state = self.state.lock().unwrap();
        if channel < state.pwm_duties.len() {
            state.pwm_duties[channel] = duty.clamp(0.0, 1.0);
        }
    }

    pub fn get_pwm_duty(&self, channel: usize) -> f32 {
        let state = self.state.lock().unwrap();
        state.pwm_duties.get(channel).copied().unwrap_or(0.0)
    }

    pub fn set_adc_reading(&self, channel: usize, val: f32) {
        let mut state = self.state.lock().unwrap();
        if channel < state.adc_readings.len() {
            state.adc_readings[channel] = val;
        }
    }

    pub fn get_adc_reading(&self, channel: usize) -> f32 {
        let state = self.state.lock().unwrap();
        state.adc_readings.get(channel).copied().unwrap_or(0.0)
    }

    pub fn set_endstop_state(&self, index: usize, triggered: bool) {
        let mut state = self.state.lock().unwrap();
        if index < state.endstops_triggered.len() {
            state.endstops_triggered[index] = triggered;
        }
    }

    pub fn is_endstop_triggered(&self, index: usize) -> bool {
        let state = self.state.lock().unwrap();
        state.endstops_triggered.get(index).copied().unwrap_or(false)
    }

    pub fn recorded_steps(&self) -> Vec<MockStepEvent> {
        self.state.lock().unwrap().step_log.clone()
    }

    pub fn total_step_count(&self) -> usize {
        self.state.lock().unwrap().step_log.len() / 2
    }
}
