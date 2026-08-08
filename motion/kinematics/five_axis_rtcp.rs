//! 5-axis RTCP engine.

use libm::{atan2f, cosf, sinf, sqrtf};

pub enum FiveAxisTopology {
    TableTable,
    HeadHead,
    HeadTable,
}

pub struct RtcpEngine {
    topology: FiveAxisTopology,
    tool_length: f32,
}

impl RtcpEngine {
    pub fn new(topology: FiveAxisTopology, tool_length: f32) -> Self {
        Self { topology, tool_length }
    }

    pub fn to_joints(&self, programmed_pos: &[f32; 3], tool_axis: &[f32; 3]) -> Result<([f32; 3], [f32; 2]), &'static str> {
        let a = atan2f(sqrtf(tool_axis[0]*tool_axis[0] + tool_axis[1]*tool_axis[1]), tool_axis[2]);
        let c = atan2f(tool_axis[1], tool_axis[0]);

        let rot_matrix = [
            [cosf(c), -sinf(c), 0.0],
            [sinf(c), cosf(c), 0.0],
            [0.0, 0.0, 1.0],
        ];

        let rot_matrix_a = [
            [1.0, 0.0, 0.0],
            [0.0, cosf(a), -sinf(a)],
            [0.0, sinf(a), cosf(a)],
        ];

        let mut r = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    r[i][j] += rot_matrix[i][k] * rot_matrix_a[k][j];
                }
            }
        }

        let tool_offset = [
            r[0][0] * 0.0 + r[0][1] * 0.0 + r[0][2] * self.tool_length,
            r[1][0] * 0.0 + r[1][1] * 0.0 + r[1][2] * self.tool_length,
            r[2][0] * 0.0 + r[2][1] * 0.0 + r[2][2] * self.tool_length,
        ];

        let linear_pos = [
            programmed_pos[0] - tool_offset[0],
            programmed_pos[1] - tool_offset[1],
            programmed_pos[2] - tool_offset[2],
        ];

        Ok((linear_pos, [a, c]))
    }
}
