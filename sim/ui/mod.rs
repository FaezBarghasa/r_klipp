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

use eframe::{egui, epi};
use crate::math::Point;

pub struct SimulatorUi {
    // Add fields for machine state, toolpath, etc.
    machine_position: Point,
}

impl Default for SimulatorUi {
    fn default() -> Self {
        Self {
            machine_position: Point::origin(),
        }
    }
}

impl epi::App for SimulatorUi {
    fn name(&self) -> &str {
        "r_klipp Simulator"
    }

    fn update(&mut self, ctx: &egui::Context, _frame: &mut epi::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("r_klipp Digital Twin");
            ui.label(format!("Machine Position: {:?}", self.machine_position));

            // 3D viewport will be rendered here using wgpu
        });
    }
}

pub fn run_ui() {
    let app = SimulatorUi::default();
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(Box::new(app), native_options);
}