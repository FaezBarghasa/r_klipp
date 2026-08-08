//! Real-time error compensation.

use crate::motion::calibration::volumetric_map::VolumetricMap;
use crate::motion::kinematics::math::se3::Transform;

pub struct Compensation<const NX: usize, const NY: usize, const NZ: usize> {
    error_map: VolumetricMap<NX, NY, NZ>,
}

impl<const NX: usize, const NY: usize, const NZ: usize> Compensation<NX, NY, NZ> {
    pub fn new(error_map: VolumetricMap<NX, NY, NZ>) -> Self {
        Self { error_map }
    }

    pub fn compensate(&self, desired_pos: &Transform) -> Transform {
        let error = self.error_map.get_error(&desired_pos.trans);

        let mut compensated_pos = *desired_pos;
        compensated_pos.trans[0] -= error[0];
        compensated_pos.trans[1] -= error[1];
        compensated_pos.trans[2] -= error[2];

        // Apply rotational error compensation (simplified)
        let da = error[3];
        let db = error[4];
        let dc = error[5];

        let rot_error_matrix = [
            [1.0, -dc, db],
            [dc, 1.0, -da],
            [-db, da, 1.0],
        ];

        let mut new_rot = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    new_rot[i][j] += compensated_pos.rot[i][k] * rot_error_matrix[k][j];
                }
            }
        }
        compensated_pos.rot = new_rot;

        compensated_pos
    }
}
