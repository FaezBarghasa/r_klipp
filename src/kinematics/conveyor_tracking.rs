
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

use crate::math::{Matrix4, Point, Vector3};
use core::sync::atomic::{AtomicI64, Ordering};

pub struct ConveyorTracker {
    /// Transformation from conveyor frame to robot base frame.
    pub conveyor_transform: Matrix4,
    /// Encoder counts per millimeter of conveyor travel.
    pub counts_per_mm: f64,
    /// Atomic variable for the current encoder count.
    pub encoder_count: &'static AtomicI64,
}

impl ConveyorTracker {
    /// Calculates the target position in the robot's base frame.
    pub fn get_target_position(&self, programmed_point: Point) -> Point {
        let conveyor_pos_mm = self.encoder_count.load(Ordering::Relaxed) as f64 / self.counts_per_mm;
        let conveyor_offset = Vector3::new(0.0, conveyor_pos_mm, 0.0);
        let conveyor_offset_transformed = self.conveyor_transform * conveyor_offset;

        programmed_point + conveyor_offset_transformed
    }

    /// Calculates the required TCP velocity to match the conveyor.
    pub fn get_target_velocity(&self, programmed_velocity: Vector3, conveyor_speed_mm_per_s: f64) -> Vector3 {
        let conveyor_velocity = Vector3::new(0.0, conveyor_speed_mm_per_s, 0.0);
        let conveyor_velocity_transformed = self.conveyor_transform.get_rotation() * conveyor_velocity;

        programmed_velocity + conveyor_velocity_transformed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_conveyor_tracking() {
        static ENCODER: AtomicI64 = AtomicI64::new(1000);
        let tracker = ConveyorTracker {
            conveyor_transform: Matrix4::from_translation(&Vector3::new(500.0, 0.0, 0.0)),
            counts_per_mm: 100.0,
            encoder_count: &ENCODER,
        };

        let programmed_point = Point::new(10.0, 20.0, 30.0);
        let target_pos = tracker.get_target_position(programmed_point);

        let conveyor_pos_mm = 1000.0 / 100.0;
        let expected_y = 20.0 + conveyor_pos_mm;
        let expected_x = 10.0 + 500.0;

        assert_eq!(target_pos.x, expected_x);
        assert_eq!(target_pos.y, 20.0); // Conveyor moves in its own Y, which is robot's X
        assert_eq!(target_pos.z, 30.0);
    }
}
