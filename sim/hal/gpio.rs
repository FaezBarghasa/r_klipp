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

use crate::hal::traits::{Adc, Gpio};

#[derive(Clone)]
pub struct VirtualGpio {
    state: Arc<AtomicBool>,
}

impl VirtualGpio {
    pub fn new(state: Arc<AtomicBool>) -> Self {
        Self { state }
    }
}

impl Gpio for VirtualGpio {
    async fn set(&mut self, state: bool) -> Result<(), &'static str> {
        self.state.store(state, Ordering::Relaxed);
        Ok(())
    }

    async fn get(&self) -> Result<bool, &'static str> {
        Ok(self.state.load(Ordering::Relaxed))
    }
}

#[derive(Clone)]
pub struct VirtualAdc {
    value: Arc<AtomicU16>,
}

impl VirtualAdc {
    pub fn new(value: Arc<AtomicU16>) -> Self {
        Self { value }
    }
}

impl Adc<u16> for VirtualAdc {
    async fn read(&mut self) -> Result<u16, &'static str> {
        Ok(self.value.load(Ordering::Relaxed))
    }
}