//! Analytical Jacobian generation from PoE parameters.

use crate::motion::kinematics::math::se3::{Transform, Twist};

pub struct Jacobian<const N: usize> {
    screws: [Twist; N],
}

impl<const N: usize> Jacobian<N> {
    pub fn new(screws: [Twist; N]) -> Self {
        Self { screws }
    }

    pub fn jacobian(&self, thetas: &[f32; N]) -> [[f32; N]; 6] {
        let mut jacobian = [[0.0; N]; 6];
        let mut t = Transform::identity();

        for i in 0..N {
            let si = &self.screws[i];
            let mut adjoint_ti = [[0.0; 6]; 6];
            let r = t.rot;
            let p = t.trans;

            adjoint_ti[0][0] = r[0][0]; adjoint_ti[0][1] = r[0][1]; adjoint_ti[0][2] = r[0][2];
            adjoint_ti[1][0] = r[1][0]; adjoint_ti[1][1] = r[1][1]; adjoint_ti[1][2] = r[1][2];
            adjoint_ti[2][0] = r[2][0]; adjoint_ti[2][1] = r[2][1]; adjoint_ti[2][2] = r[2][2];

            adjoint_ti[3][3] = r[0][0]; adjoint_ti[3][4] = r[0][1]; adjoint_ti[3][5] = r[0][2];
            adjoint_ti[4][3] = r[1][0]; adjoint_ti[4][4] = r[1][1]; adjoint_ti[4][5] = r[1][2];
            adjoint_ti[5][3] = r[2][0]; adjoint_ti[5][4] = r[2][1]; adjoint_ti[5][5] = r[2][2];

            let p_skew = [
                [0.0, -p[2], p[1]],
                [p[2], 0.0, -p[0]],
                [-p[1], p[0], 0.0],
            ];

            let pr = [
                [
                    p_skew[0][0]*r[0][0] + p_skew[0][1]*r[1][0] + p_skew[0][2]*r[2][0],
                    p_skew[0][0]*r[0][1] + p_skew[0][1]*r[1][1] + p_skew[0][2]*r[2][1],
                    p_skew[0][0]*r[0][2] + p_skew[0][1]*r[1][2] + p_skew[0][2]*r[2][2],
                ],
                [
                    p_skew[1][0]*r[0][0] + p_skew[1][1]*r[1][0] + p_skew[1][2]*r[2][0],
                    p_skew[1][0]*r[0][1] + p_skew[1][1]*r[1][1] + p_skew[1][2]*r[2][1],
                    p_skew[1][0]*r[0][2] + p_skew[1][1]*r[1][2] + p_skew[1][2]*r[2][2],
                ],
                [
                    p_skew[2][0]*r[0][0] + p_skew[2][1]*r[1][0] + p_skew[2][2]*r[2][0],
                    p_skew[2][0]*r[0][1] + p_skew[2][1]*r[1][1] + p_skew[2][2]*r[2][1],
                    p_skew[2][0]*r[0][2] + p_skew[2][1]*r[1][2] + p_skew[2][2]*r[2][2],
                ],
            ];

            adjoint_ti[3][0] = pr[0][0]; adjoint_ti[3][1] = pr[0][1]; adjoint_ti[3][2] = pr[0][2];
            adjoint_ti[4][0] = pr[1][0]; adjoint_ti[4][1] = pr[1][1]; adjoint_ti[4][2] = pr[1][2];
            adjoint_ti[5][0] = pr[2][0]; adjoint_ti[5][1] = pr[2][1]; adjoint_ti[5][2] = pr[2][2];

            let ji = [
                si.v[0], si.v[1], si.v[2],
                si.w[0], si.w[1], si.w[2],
            ];

            for row in 0..6 {
                jacobian[row][i] = adjoint_ti[row][0] * ji[0] + adjoint_ti[row][1] * ji[1] + adjoint_ti[row][2] * ji[2]
                                + adjoint_ti[row][3] * ji[3] + adjoint_ti[row][4] * ji[4] + adjoint_ti[row][5] * ji[5];
            }

            t = t * crate::motion::kinematics::math::se3::exponential_map(si, thetas[i]);
        }

        jacobian
    }
}
