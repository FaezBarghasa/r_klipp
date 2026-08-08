
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

#![allow(non_snake_case)]

use crate::math::{Point, Vector3};
use libm::{atan2, cos, sin, sqrt};

pub enum FiveAxisTopology {
    TableTable, // BC on table
    HeadHead,   // BC on head
    HeadTable,  // B on head, C on table
}

pub struct RtcpKinematics {
    pub topology: FiveAxisTopology,
    pub tool_length: f64,
}

impl RtcpKinematics {
    /// Calculates the machine axes positions for a given tool tip position and orientation.
    pub fn inverse(
        &self,
        programmed_point: Point,
        tool_axis: Vector3,
    ) -> Result<(Point, f64, f64), &'static str> {
        let tool_axis = tool_axis.normalize();
        let (b, c) = self.calculate_rotary_angles(tool_axis);

        let R_b = Self::rotation_matrix_y(b);
        let R_c = Self::rotation_matrix_z(c);
        let R_rotary = R_c * R_b;

        let tool_offset = R_rotary * Vector3::new(0.0, 0.0, self.tool_length);

        let linear_axes = match self.topology {
            FiveAxisTopology::TableTable => {
                let inv_R_rotary = R_rotary.transpose();
                inv_R_rotary * (programmed_point.to_vector() - tool_offset)
            }
            FiveAxisTopology::HeadHead => programmed_point.to_vector() - tool_offset,
            FiveAxisTopology::HeadTable => {
                let inv_R_c = R_c.transpose();
                inv_R_c * (programmed_point.to_vector() - tool_offset)
            }
        };

        Ok((linear_axes.into(), b, c))
    }

    /// Calculates the forward kinematics: from machine axes to TCP position.
    pub fn forward(&self, linear_axes: Point, b: f64, c: f64) -> Point {
        let R_b = Self::rotation_matrix_y(b);
        let R_c = Self::rotation_matrix_z(c);
        let R_rotary = R_c * R_b;

        let tool_offset = R_rotary * Vector3::new(0.0, 0.0, self.tool_length);

        match self.topology {
            FiveAxisTopology::TableTable => {
                (R_rotary * linear_axes.to_vector() + tool_offset).into()
            }
            FiveAxisTopology::HeadHead => (linear_axes.to_vector() + tool_offset).into(),
            FiveAxisTopology::HeadTable => (R_c * linear_axes.to_vector() + tool_offset).into(),
        }
    }

    fn calculate_rotary_angles(&self, tool_axis: Vector3) -> (f64, f64) {
        let b = atan2(sqrt(tool_axis.x * tool_axis.x + tool_axis.y * tool_axis.y), tool_axis.z);
        let c = atan2(tool_axis.y, tool_axis.x);
        (b, c)
    }

    fn rotation_matrix_y(angle: f64) -> crate::math::Matrix3 {
        crate::math::Matrix3::from_rows(
            [cos(angle), 0.0, sin(angle)],
            [0.0, 1.0, 0.0],
            [-sin(angle), 0.0, cos(angle)],
        )
    }

    fn rotation_matrix_z(angle: f64) -> crate::math::Matrix3 {
        crate::math::Matrix3::from_rows(
            [cos(angle), -sin(angle), 0.0],
            [sin(angle), cos(angle), 0.0],
            [0.0, 0.0, 1.0],
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rtcp_head_head() {
        let kinematics = RtcpKinematics {
            topology: FiveAxisTopology::HeadHead,
            tool_length: 100.0,
        };
        let programmed_point = Point::new(50.0, 50.0, 0.0);
        let tool_axis = Vector3::new(0.0, 0.5, sqrt(0.75)).normalize(); // 30 degrees tilt in Y

        let (linear_axes, b, c) = kinematics.inverse(programmed_point, tool_axis).unwrap();
        let tcp_pos = kinematics.forward(linear_axes, b, c);

        assert!((tcp_pos.x - programmed_point.x).abs() < 1e-9);
        assert!((tcp_pos.y - programmed_point.y).abs() < 1e-9);
        assert!((tcp_pos.z - programmed_point.z).abs() < 1e-9);
    }
}
