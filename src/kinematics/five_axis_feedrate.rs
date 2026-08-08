
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

use crate::kinematics::five_axis_rtcp::RtcpKinematics;
use crate::math::Point;

pub struct FeedrateScheduler {
    pub max_rotary_velocity: f64, // degrees per minute
}

impl FeedrateScheduler {
    /// Schedules the feedrate for a 5-axis move, ensuring rotary axis limits are not exceeded.
    pub fn schedule_feedrate(
        &self,
        kinematics: &RtcpKinematics,
        p1: Point,
        p2: Point,
        tool_axis1: crate::math::Vector3,
        tool_axis2: crate::math::Vector3,
        programmed_feedrate: f64,
    ) -> Result<f64, &'static str> {
        let (linear1, b1, c1) = kinematics.inverse(p1, tool_axis1)?;
        let (linear2, b2, c2) = kinematics.inverse(p2, tool_axis2)?;

        let linear_dist = (linear2 - linear1).norm();
        let b_dist = (b2 - b1).to_degrees();
        let c_dist = (c2 - c1).to_degrees();

        let time_linear = linear_dist / programmed_feedrate;
        let time_b = b_dist.abs() / self.max_rotary_velocity;
        let time_c = c_dist.abs() / self.max_rotary_velocity;

        let required_time = time_linear.max(time_b).max(time_c);

        if required_time > time_linear {
            // Rotary axes are the limiting factor, scale down the linear feedrate.
            Ok(linear_dist / required_time)
        } else {
            Ok(programmed_feedrate)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kinematics::five_axis_rtcp::FiveAxisTopology;
    use crate::math::Vector3;
    use libm::sqrt;

    #[test]
    fn test_feedrate_scheduling() {
        let scheduler = FeedrateScheduler {
            max_rotary_velocity: 1800.0, // 5 RPM
        };
        let kinematics = RtcpKinematics {
            topology: FiveAxisTopology::HeadHead,
            tool_length: 100.0,
        };

        let p1 = Point::new(0.0, 0.0, 0.0);
        let tool_axis1 = Vector3::new(0.0, 0.0, 1.0);

        let p2 = Point::new(10.0, 0.0, 0.0);
        // This move requires a large rotation of the B-axis
        let tool_axis2 = Vector3::new(0.0, 0.866, 0.5); // 60 degrees tilt

        let programmed_feedrate = 1000.0; // mm/min

        let scheduled_feedrate = scheduler
            .schedule_feedrate(&kinematics, p1, p2, tool_axis1, tool_axis2, programmed_feedrate)
            .unwrap();

        // The feedrate should be scaled down because of the fast rotary move.
        assert!(scheduled_feedrate < programmed_feedrate);
    }
}
