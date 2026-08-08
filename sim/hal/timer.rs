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

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

#[derive(Clone)]
pub struct VirtualClock {
    start_time: Instant,
    time_scale: Arc<Mutex<f64>>,
}

impl VirtualClock {
    pub fn new(time_scale: Arc<Mutex<f64>>) -> Self {
        Self {
            start_time: Instant::now(),
            time_scale,
        }
    }

    pub fn now_secs_f64(&self) -> f64 {
        let scale = *self.time_scale.lock().unwrap();
        self.start_time.elapsed().as_secs_f64() * scale
    }
}

pub struct VirtualTimer {
    clock: VirtualClock,
    deadline: f64,
}

impl VirtualTimer {
    pub fn new(clock: VirtualClock, duration_secs: f64) -> Self {
        let deadline = clock.now_secs_f64() + duration_secs;
        Self { clock, deadline }
    }

    pub async fn wait(&self) {
        while self.clock.now_secs_f64() < self.deadline {
            tokio::time::sleep(Duration::from_micros(100)).await;
        }
    }
}