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

#[derive(Clone, Copy)]
pub struct State {
    pub p: f64,
    pub v: f64,
    pub a: f64,
    pub j: f64,
}

pub struct Extrapolator {
    last_state: State,
    last_time: f64,
}

impl Extrapolator {
    pub fn new() -> Self {
        Self {
            last_state: State { p: 0.0, v: 0.0, a: 0.0, j: 0.0 },
            last_time: 0.0,
        }
    }

    pub fn update_plc_state(&mut self, state: State, time: f64) {
        self.last_state = state;
        self.last_time = time;
    }

    pub fn get_servo_state(&self, servo_time: f64) -> (f64, f64) {
        let dt = servo_time - self.last_time;
        let s = &self.last_state;
        let p1 = s.p + s.v * dt + 0.5 * s.a * dt * dt + (1.0 / 6.0) * s.j * dt * dt * dt;
        let v1 = s.v + s.a * dt + 0.5 * s.j * dt * dt;
        (p1, v1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extrapolator() {
        let mut extrapolator = Extrapolator::new();
        let plc_state = State { p: 0.0, v: 10.0, a: 2.0, j: 0.5 };
        extrapolator.update_plc_state(plc_state, 0.0);

        let (p1, v1) = extrapolator.get_servo_state(0.1);
        assert!((p1 - (0.0 + 10.0*0.1 + 0.5*2.0*0.1*0.1 + 1.0/6.0*0.5*0.1*0.1*0.1)).abs() < 1e-6);
        assert!((v1 - (10.0 + 2.0*0.1 + 0.5*0.5*0.1*0.1)).abs() < 1e-6);
    }
}