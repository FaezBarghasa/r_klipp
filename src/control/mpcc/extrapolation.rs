
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

/// Extrapolates position and velocity using a 3rd-order polynomial.
pub fn extrapolate_state(
    p0: f64,
    v0: f64,
    a0: f64,
    j0: f64,
    dt: f64,
) -> (f64, f64) {
    let p1 = p0 + v0 * dt + 0.5 * a0 * dt * dt + (1.0 / 6.0) * j0 * dt * dt * dt;
    let v1 = v0 + a0 * dt + 0.5 * j0 * dt * dt;
    (p1, v1)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extrapolation() {
        let (p1, v1) = extrapolate_state(0.0, 10.0, 2.0, 0.5, 0.1);
        assert!((p1 - 1.01008333).abs() < 1e-6);
        assert!((v1 - 10.2025).abs() < 1e-6);
    }
}
