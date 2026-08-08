use crate::hal::gpio::VirtualPin;
use rapier3d::prelude::*;

pub struct VirtualEndstop {
    pin: VirtualPin,
    limit: f32,
    axis: u8,
}

impl VirtualEndstop {
    pub fn new(pin: VirtualPin, limit: f32, axis: u8) -> Self {
        Self { pin, limit, axis }
    }

    pub fn update(&mut self, position: &Vector<f32>) {
        if position[self.axis as usize] >= self.limit {
            self.pin.set_high().unwrap();
        } else {
            self.pin.set_low().unwrap();
        }
    }
}

pub struct VirtualProbe {
    pin: VirtualPin,
    workpiece: ColliderHandle,
}

impl VirtualProbe {
    pub fn new(pin: VirtualPin, workpiece: ColliderHandle) -> Self {
        Self { pin, workpiece }
    }

    pub fn update(&mut self, tool: ColliderHandle, query_pipeline: &QueryPipeline) {
        if query_pipeline.intersection_pair(self.workpiece, tool).is_some() {
            self.pin.set_high().unwrap();
        } else {
            self.pin.set_low().unwrap();
        }
    }
}

pub struct VirtualThermodynamics {
    adc_channel: crate::hal::adc::VirtualAdcChannel,
    pwm_channel: crate::hal::pwm::VirtualPwmChannel,
    thermal_mass: f32,
    dissipation: f32,
    temperature: f32,
}

impl VirtualThermodynamics {
    pub fn new(
        adc_channel: crate::hal::adc::VirtualAdcChannel,
        pwm_channel: crate::hal::pwm::VirtualPwmChannel,
        thermal_mass: f32,
        dissipation: f32,
    ) -> Self {
        Self {
            adc_channel,
            pwm_channel,
            thermal_mass,
            dissipation,
            temperature: 25.0,
        }
    }

    pub fn update(&mut self, dt: f32) {
        let power_in = self.pwm_channel.get_duty_cycle() * 100.0; // 100W heater
        let power_out = (self.temperature - 25.0) * self.dissipation;
        let net_power = power_in - power_out;
        self.temperature += net_power * dt / self.thermal_mass;
        self.adc_channel.set_value(self.temperature);
    }
}
