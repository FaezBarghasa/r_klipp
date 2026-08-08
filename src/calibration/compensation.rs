
// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law of a greed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use crate::calibration::volumetric_map::VolumetricMap;
use crate::math::Point;

/// Applies volumetric error compensation to a desired point.
pub fn compensate_point(
    desired_point: Point,
    volumetric_map: &VolumetricMap,
) -> Point {
    let error = volumetric_map.evaluate_error(desired_point);
    desired_point - error
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_compensation() {
        let map = VolumetricMap { _data: [0.0] };
        let p = Point::new(10.0, 20.0, 30.0);
        let compensated_p = compensate_point(p, &map);
        // Since the placeholder map returns zero error, the point should be unchanged.
        assert_eq!(p, compensated_p);
    }
}
