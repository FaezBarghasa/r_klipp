//! Virtual Sensors for Endstops, Probes, and Thermal monitoring.

use crate::hal::MockHal;

pub struct VirtualEndstopSensor {
    pub axis_index: usize,
    pub trigger_position_mm: f64,
}

impl VirtualEndstopSensor {
    pub fn new(axis_index: usize, trigger_position_mm: f64) -> Self {
        Self {
            axis_index,
            trigger_position_mm,
        }
    }

    pub fn update(&self, current_pos_mm: &[f64; 3], hal: &MockHal) {
        if self.axis_index < current_pos_mm.len() {
            let triggered = current_pos_mm[self.axis_index] <= self.trigger_position_mm;
            hal.set_endstop_state(self.axis_index, triggered);
        }
    }
}

pub struct VirtualThermodynamicsSensor {
    pub adc_channel: usize,
    pub pwm_channel: usize,
    pub thermal_mass: f32,
    pub dissipation: f32,
    pub temperature_c: f32,
}

impl VirtualThermodynamicsSensor {
    pub fn new(adc_channel: usize, pwm_channel: usize, thermal_mass: f32, dissipation: f32) -> Self {
        Self {
            adc_channel,
            pwm_channel,
            thermal_mass: thermal_mass.max(0.1),
            dissipation: dissipation.max(0.01),
            temperature_c: 22.0,
        }
    }

    pub fn update(&mut self, dt_s: f32, hal: &MockHal) {
        let duty = hal.get_pwm_duty(self.pwm_channel);
        let power_in = duty * 50.0; // 50W heater cartridge
        let power_out = (self.temperature_c - 22.0) * self.dissipation;
        let net_power = power_in - power_out;

        self.temperature_c += (net_power / self.thermal_mass) * dt_s;
        hal.set_adc_reading(self.adc_channel, self.temperature_c);
    }
}
