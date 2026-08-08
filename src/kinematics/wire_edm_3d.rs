
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

use crate::math::{Point, Vector3};

pub struct WireEdm3d {
    pub wire_radius: f64,
    pub spark_gap: f64,
}

impl WireEdm3d {
    /// Calculates the required upper and lower guide positions for a 3D tapered cut.
    pub fn calculate_guide_positions(
        &self,
        programmed_point: Point,
        cut_direction: Vector3,
        wire_tilt_vector: Vector3,
    ) -> (Point, Point) {
        let total_offset = self.wire_radius + self.spark_gap;
        let wire_direction = wire_tilt_vector.normalize();
        let cut_direction = cut_direction.normalize();

        // The offset vector is perpendicular to both the wire and the cut direction.
        let offset_vector = (wire_direction.cross(cut_direction))
            .cross(wire_direction)
            .normalize();

        let offset = offset_vector * total_offset;
        let contact_point = programmed_point + offset;

        // Assuming the programmed point is at Z=0 on the workpiece top surface.
        // And the wire is tilted around this point.
        let lower_guide_z = -100.0; // Example Z height for lower guide
        let upper_guide_z = 100.0;  // Example Z height for upper guide

        let lower_guide = contact_point + wire_direction * (lower_guide_z / wire_direction.z);
        let upper_guide = contact_point + wire_direction * (upper_guide_z / wire_direction.z);

        (lower_guide, upper_guide)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_3d_wire_offset() {
        let edm = WireEdm3d {
            wire_radius: 0.1,
            spark_gap: 0.05,
        };

        let programmed_point = Point::new(10.0, 10.0, 0.0);
        let cut_direction = Vector3::new(1.0, 0.0, 0.0);
        // 15 degrees tilt in Y
        let wire_tilt_vector = Vector3::new(0.0, libm::sin(15.0f64.to_radians()), libm::cos(15.0f64.to_radians()));

        let (lower, upper) =
            edm.calculate_guide_positions(programmed_point, cut_direction, wire_tilt_vector);

        // The X coordinate of the guides should be offset from the programmed point.
        assert!((lower.x - programmed_point.x).abs() > 0.1);
        assert!((upper.x - programmed_point.x).abs() > 0.1);

        // The Y coordinate should also be slightly offset due to the 3D geometry.
        assert!((lower.y - programmed_point.y).abs() > 1e-3);
    }
}
