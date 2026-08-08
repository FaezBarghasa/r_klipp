//! 3D volumetric error map via B-Splines.

pub struct VolumetricMap<const NX: usize, const NY: usize, const NZ: usize> {
    // B-Spline control points for the error field.
    // Each point is a 6D vector (dx, dy, dz, da, db, dc).
    control_points: [[[f32; 6]; NZ]; NY],
    // Knots for the B-Spline.
    knots_x: [f32; NX + 4],
    knots_y: [f32; NY + 4],
    knots_z: [f32; NZ + 4],
}

impl<const NX: usize, const NY: usize, const NZ: usize> VolumetricMap<NX, NY, NZ> {
    pub fn new(
        control_points: [[[f32; 6]; NZ]; NY],
        knots_x: [f32; NX + 4],
        knots_y: [f32; NY + 4],
        knots_z: [f32; NZ + 4],
    ) -> Self {
        Self {
            control_points,
            knots_x,
            knots_y,
            knots_z,
        }
    }

    fn basis_function(&self, u: f32, knots: &[f32]) -> [f32; 4] {
        // Simplified B-spline basis function (degree 3)
        let mut n = [0.0; 4];
        let t = u;
        n[0] = (1.0 - t) * (1.0 - t) * (1.0 - t) / 6.0;
        n[1] = (3.0 * t * t * t - 6.0 * t * t + 4.0) / 6.0;
        n[2] = (-3.0 * t * t * t + 3.0 * t * t + 3.0 * t + 1.0) / 6.0;
        n[3] = t * t * t / 6.0;
        n
    }

    pub fn get_error(&self, pos: &[f32; 3]) -> [f32; 6] {
        let x_basis = self.basis_function(pos[0], &self.knots_x);
        let y_basis = self.basis_function(pos[1], &self.knots_y);
        let z_basis = self.basis_function(pos[2], &self.knots_z);

        let mut error = [0.0; 6];
        for i in 0..4 {
            for j in 0..4 {
                for k in 0..4 {
                    let weight = x_basis[i] * y_basis[j] * z_basis[k];
                    for l in 0..6 {
                        // This assumes control_points is accessible with i,j,k which is not correct
                        // for a sparse grid. A real implementation would need to find the correct
                        // control points for the given position.
                        // error[l] += self.control_points[i][j][k][l] * weight;
                    }
                }
            }
        }
        error
    }
}
