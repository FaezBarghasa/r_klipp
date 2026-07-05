// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;

use crate::math::Point;

pub struct VirtualEndstop {
    position: Arc<Point>,
    limit: f64,
    axis: usize,
    triggered: Arc<AtomicBool>,
}

impl VirtualEndstop {
    pub fn new(position: Arc<Point>, limit: f64, axis: usize, triggered: Arc<AtomicBool>) -> Self {
        Self {
            position,
            limit,
            axis,
            triggered,
        }
    }

    pub fn update(&self) {
        let pos = match self.axis {
            0 => self.position.x,
            1 => self.position.y,
            2 => self.position.z,
            _ => 0.0,
        };
        if pos >= self.limit {
            self.triggered.store(true, Ordering::Relaxed);
        } else {
            self.triggered.store(false, Ordering::Relaxed);
        }
    }
}

pub struct VirtualHeater {
    temperature: Arc<AtomicU16>,
    power: Arc<AtomicBool>,
    thermal_mass: f32,
    dissipation: f32,
}

impl VirtualHeater {
    pub fn new(
        temperature: Arc<AtomicU16>,
        power: Arc<AtomicBool>,
        thermal_mass: f32,
        dissipation: f32,
    ) -> Self {
        Self {
            temperature,
            power,
            thermal_mass,
            dissipation,
        }
    }

    pub fn update(&self, dt: f32) {
        let mut temp = self.temperature.load(Ordering::Relaxed) as f32;
        if self.power.load(Ordering::Relaxed) {
            temp += 100.0 / self.thermal_mass * dt; // 100W heater
        }
        temp -= (temp - 25.0) * self.dissipation * dt; // Dissipate to 25C ambient
        self.temperature.store(temp as u16, Ordering::Relaxed);
    }
}