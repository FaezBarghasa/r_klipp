//! Jacobian Derivative (Hessian) for C^3 continuous motion.

use crate::motion::kinematics::math::se3::{Twist, Transform};
use crate::motion::kinematics::jacobian::Jacobian;

pub struct Hessian<const N: usize> {
    screws: [Twist; N],
    jacobian_calculator: Jacobian<N>,
}

impl<const N: usize> Hessian<N> {
    pub fn new(screws: [Twist; N]) -> Self {
        Self {
            screws,
            jacobian_calculator: Jacobian::new(screws),
        }
    }

    pub fn dot_jacobian(&self, thetas: &[f32; N], dot_thetas: &[f32; N]) -> [[f32; N]; 6] {
        let mut dot_j = [[0.0; N]; 6];
        let j = self.jacobian_calculator.jacobian(thetas);

        for i in 0..N {
            let mut ad_ji_dot_qi = [0.0; 6];
            let mut sum_k_1_to_i_minus_1 = [0.0; 6];

            for k in 0..i {
                let jk = [j[0][k], j[1][k], j[2][k], j[3][k], j[4][k], j[5][k]];
                sum_k_1_to_i_minus_1[0] += jk[0] * dot_thetas[k];
                sum_k_1_to_i_minus_1[1] += jk[1] * dot_thetas[k];
                sum_k_1_to_i_minus_1[2] += jk[2] * dot_thetas[k];
                sum_k_1_to_i_minus_1[3] += jk[3] * dot_thetas[k];
                sum_k_1_to_i_minus_1[4] += jk[4] * dot_thetas[k];
                sum_k_1_to_i_minus_1[5] += jk[5] * dot_thetas[k];
            }

            let ji = [j[0][i], j[1][i], j[2][i], j[3][i], j[4][i], j[5][i]];

            let v_sum = [sum_k_1_to_i_minus_1[0], sum_k_1_to_i_minus_1[1], sum_k_1_to_i_minus_1[2]];
            let w_sum = [sum_k_1_to_i_minus_1[3], sum_k_1_to_i_minus_1[4], sum_k_1_to_i_minus_1[5]];
            let v_i = [ji[0], ji[1], ji[2]];
            let w_i = [ji[3], ji[4], ji[5]];

            let cross_v_sum_v_i = [
                v_sum[1] * v_i[2] - v_sum[2] * v_i[1],
                v_sum[2] * v_i[0] - v_sum[0] * v_i[2],
                v_sum[0] * v_i[1] - v_sum[1] * v_i[0],
            ];

            let cross_w_sum_v_i = [
                w_sum[1] * v_i[2] - w_sum[2] * v_i[1],
                w_sum[2] * v_i[0] - w_sum[0] * v_i[2],
                w_sum[0] * v_i[1] - w_sum[1] * v_i[0],
            ];

            let cross_w_sum_w_i = [
                w_sum[1] * w_i[2] - w_sum[2] * w_i[1],
                w_sum[2] * w_i[0] - w_sum[0] * w_i[2],
                w_sum[0] * w_i[1] - w_sum[1] * w_i[0],
            ];

            ad_ji_dot_qi[0] = cross_v_sum_v_i[0] + cross_w_sum_v_i[0];
            ad_ji_dot_qi[1] = cross_v_sum_v_i[1] + cross_w_sum_v_i[1];
            ad_ji_dot_qi[2] = cross_v_sum_v_i[2] + cross_w_sum_v_i[2];
            ad_ji_dot_qi[3] = cross_w_sum_w_i[0];
            ad_ji_dot_qi[4] = cross_w_sum_w_i[1];
            ad_ji_dot_qi[5] = cross_w_sum_w_i[2];

            for row in 0..6 {
                dot_j[row][i] = ad_ji_dot_qi[row];
            }
        }

        dot_j
    }
}
