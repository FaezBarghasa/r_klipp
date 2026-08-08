//! Frenet-Serret frame and contour error calculation.

use libm::sqrtf;

pub struct ContourError {
    // Path is represented by a function that gives position and derivatives.
    // This is a simplification. A real implementation would use a path object.
}

impl ContourError {
    pub fn new() -> Self {
        Self {}
    }

    fn closest_point_on_path(&self, actual_pos: &[f32; 3], path_u: f32) -> f32 {
        // Simplified: assumes the closest point is near the current path parameter.
        // A real implementation would use Newton-Raphson or another search.
        path_u
    }

    pub fn calculate_error(&self, actual_pos: &[f32; 3], path_u: f32, path_ders: &[[f32; 3]; 3]) -> (f32, [f32; 3]) {
        let closest_u = self.closest_point_on_path(actual_pos, path_u);
        let c_prime = &path_ders[1];
        let c_double_prime = &path_ders[2];

        let t_vec = {
            let norm = sqrtf(c_prime[0]*c_prime[0] + c_prime[1]*c_prime[1] + c_prime[2]*c_prime[2]);
            [c_prime[0]/norm, c_prime[1]/norm, c_prime[2]/norm]
        };

        let n_vec = {
            let cross_product = [
                c_prime[1] * c_double_prime[2] - c_prime[2] * c_double_prime[1],
                c_prime[2] * c_double_prime[0] - c_prime[0] * c_double_prime[2],
                c_prime[0] * c_double_prime[1] - c_prime[1] * c_double_prime[0],
            ];
            let cross_norm = sqrtf(cross_product[0]*cross_product[0] + cross_product[1]*cross_product[1] + cross_product[2]*cross_product[2]);
            let c_prime_norm = sqrtf(c_prime[0]*c_prime[0] + c_prime[1]*c_prime[1] + c_prime[2]*c_prime[2]);
            let curvature = cross_norm / (c_prime_norm * c_prime_norm * c_prime_norm);

            let acc = [
                c_double_prime[0] - (c_prime[0] * (c_prime[0]*c_double_prime[0] + c_prime[1]*c_double_prime[1] + c_prime[2]*c_double_prime[2]) / (c_prime_norm*c_prime_norm)),
                c_double_prime[1] - (c_prime[1] * (c_prime[0]*c_double_prime[0] + c_prime[1]*c_double_prime[1] + c_prime[2]*c_double_prime[2]) / (c_prime_norm*c_prime_norm)),
                c_double_prime[2] - (c_prime[2] * (c_prime[0]*c_double_prime[0] + c_prime[1]*c_double_prime[1] + c_prime[2]*c_double_prime[2]) / (c_prime_norm*c_prime_norm)),
            ];
            let acc_norm = sqrtf(acc[0]*acc[0] + acc[1]*acc[1] + acc[2]*acc[2]);
            [acc[0]/acc_norm, acc[1]/acc_norm, acc[2]/acc_norm]
        };

        let b_vec = [
            t_vec[1] * n_vec[2] - t_vec[2] * n_vec[1],
            t_vec[2] * n_vec[0] - t_vec[0] * n_vec[2],
            t_vec[0] * n_vec[1] - t_vec[1] * n_vec[0],
        ];

        let pos_error = [
            actual_pos[0] - path_ders[0][0],
            actual_pos[1] - path_ders[0][1],
            actual_pos[2] - path_ders[0][2],
        ];

        let contour_error_n = pos_error[0]*n_vec[0] + pos_error[1]*n_vec[1] + pos_error[2]*n_vec[2];
        let contour_error_b = pos_error[0]*b_vec[0] + pos_error[1]*b_vec[1] + pos_error[2]*b_vec[2];

        let contour_error_vec = [
            contour_error_n * n_vec[0] + contour_error_b * b_vec[0],
            contour_error_n * n_vec[1] + contour_error_b * b_vec[1],
            contour_error_n * n_vec[2] + contour_error_b * b_vec[2],
        ];

        (sqrtf(contour_error_n*contour_error_n + contour_error_b*contour_error_b), contour_error_vec)
    }
}
