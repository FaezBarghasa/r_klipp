//! 3D spatial wire offset for tapered EDM.

use libm::sqrtf;

pub struct WireEdm3d {
    gap_distance: f32,
}

impl WireEdm3d {
    pub fn new(gap_distance: f32) -> Self {
        Self { gap_distance }
    }

    pub fn offset(&self, lower_guide: &[f32; 3], upper_guide: &[f32; 3], cut_direction: &[f32; 3]) -> ([f32; 3], [f32; 3]) {
        let wire_vec = [
            upper_guide[0] - lower_guide[0],
            upper_guide[1] - lower_guide[1],
            upper_guide[2] - lower_guide[2],
        ];

        let cross_wire_cut = [
            wire_vec[1] * cut_direction[2] - wire_vec[2] * cut_direction[1],
            wire_vec[2] * cut_direction[0] - wire_vec[0] * cut_direction[2],
            wire_vec[0] * cut_direction[1] - wire_vec[1] * cut_direction[0],
        ];

        let cross_final = [
            cross_wire_cut[1] * wire_vec[2] - cross_wire_cut[2] * wire_vec[1],
            cross_wire_cut[2] * wire_vec[0] - cross_wire_cut[0] * wire_vec[2],
            cross_wire_cut[0] * wire_vec[1] - cross_wire_cut[1] * wire_vec[0],
        ];

        let norm = sqrtf(cross_final[0]*cross_final[0] + cross_final[1]*cross_final[1] + cross_final[2]*cross_final[2]);
        let offset_vec = [
            cross_final[0] / norm * self.gap_distance,
            cross_final[1] / norm * self.gap_distance,
            cross_final[2] / norm * self.gap_distance,
        ];

        let new_lower = [
            lower_guide[0] + offset_vec[0],
            lower_guide[1] + offset_vec[1],
            lower_guide[2] + offset_vec[2],
        ];

        let new_upper = [
            upper_guide[0] + offset_vec[0],
            upper_guide[1] + offset_vec[1],
            upper_guide[2] + offset_vec[2],
        ];

        (new_lower, new_upper)
    }
}
