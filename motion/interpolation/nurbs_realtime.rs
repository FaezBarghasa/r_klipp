//! NURBS interpolation with second-order Taylor series expansion.

use libm::sqrtf;

pub struct NurbsRealtime<const DEGREE: usize, const CTRL_PTS: usize> {
    control_points: [[f32; 4]; CTRL_PTS],
    knots: [f32; CTRL_PTS + DEGREE + 1],
}

impl<const DEGREE: usize, const CTRL_PTS: usize> NurbsRealtime<DEGREE, CTRL_PTS> {
    pub fn new(control_points: [[f32; 4]; CTRL_PTS], knots: [f32; CTRL_PTS + DEGREE + 1]) -> Self {
        Self { control_points, knots }
    }

    fn find_span(&self, u: f32) -> usize {
        // Linear search, for embedded systems a binary search might be better
        // if the number of knots is large.
        for i in DEGREE..(CTRL_PTS) {
            if u >= self.knots[i] && u < self.knots[i + 1] {
                return i;
            }
        }
        CTRL_PTS - 1
    }

    fn basis_funs(&self, span: usize, u: f32) -> [[f32; DEGREE + 1]; DEGREE + 1] {
        let mut n = [[0.0; DEGREE + 1]; DEGREE + 1];
        let mut left = [0.0; DEGREE + 1];
        let mut right = [0.0; DEGREE + 1];

        n[0][0] = 1.0;

        for j in 1..=DEGREE {
            left[j] = u - self.knots[span + 1 - j];
            right[j] = self.knots[span + j] - u;
            let mut saved = 0.0;

            for r in 0..j {
                let temp = n[r][j - 1] / (right[r + 1] + left[j - r]);
                n[r][j] = saved + right[r + 1] * temp;
                saved = left[j - r] * temp;
            }
            n[j][j] = saved;
        }
        n
    }

    fn ders_basis_funs(&self, span: usize, u: f32, n_ders: usize) -> [[[f32; DEGREE + 1]; DEGREE + 1]; 2] {
        let mut ndu = [[[0.0; DEGREE + 1]; DEGREE + 1]; 2];
        let mut left = [0.0; DEGREE + 1];
        let mut right = [0.0; DEGREE + 1];

        ndu[0][0][0] = 1.0;

        for j in 1..=DEGREE {
            left[j] = u - self.knots[span + 1 - j];
            right[j] = self.knots[span + j] - u;
            let mut saved = 0.0;

            for r in 0..j {
                ndu[j][r][0] = right[r + 1] + left[j - r];
                let tmp = ndu[r][j - 1][0] / ndu[j][r][0];
                ndu[r][j][0] = saved + right[r + 1] * tmp;
                saved = left[j - r] * tmp;
            }
            ndu[j][j][0] = saved;
        }

        for r in 0..=DEGREE {
            let mut s1 = 0;
            let mut s2 = 1;
            let mut a = [[0.0; DEGREE + 1]; 2];
            a[0][0] = 1.0;

            for k in 1..=n_ders {
                let mut d = 0.0;
                let rk = r - k;
                let pk = DEGREE - k;
                if r >= k {
                    a[s2][0] = a[s1][0] / ndu[pk + 1][rk][0];
                    d = a[s2][0] * ndu[rk][pk][0];
                }
                let j1 = if rk >= -1 { 1 } else { -rk };
                let j2 = if r - 1 <= pk { k - 1 } else { DEGREE - r };

                for j in j1..=j2 {
                    a[s2][j] = (a[s1][j] - a[s1][j - 1]) / ndu[pk + 1][rk + j][0];
                    d += a[s2][j] * ndu[rk + j][pk][0];
                }
                if r <= pk {
                    a[s2][k] = -a[s1][k - 1] / ndu[pk + 1][r][0];
                    d += a[s2][k] * ndu[r][pk][0];
                }
                ndu[r][DEGREE][k] = d;
                let temp = s1;
                s1 = s2;
                s2 = temp;
            }
        }

        let mut r = DEGREE as f32;
        for k in 1..=n_ders {
            for j in 0..=DEGREE {
                ndu[j][DEGREE][k] *= r;
            }
            r *= (DEGREE - k) as f32;
        }

        ndu
    }


    pub fn point_and_ders(&self, u: f32, num_ders: usize) -> [[f32; 4]; 2] {
        let span = self.find_span(u);
        let ders = self.ders_basis_funs(span, u, num_ders);
        let mut result = [[0.0; 4]; 2];

        for k in 0..=num_ders {
            for i in 0..=DEGREE {
                let idx = span - DEGREE + i;
                result[k][0] += ders[i][DEGREE][k] * self.control_points[idx][0];
                result[k][1] += ders[i][DEGREE][k] * self.control_points[idx][1];
                result[k][2] += ders[i][DEGREE][k] * self.control_points[idx][2];
                result[k][3] += ders[i][DEGREE][k] * self.control_points[idx][3];
            }
        }
        result
    }

    pub fn update_param(&self, u: f32, v: f32, t: f32) -> f32 {
        let c_ders = self.point_and_ders(u, 2);
        let c_prime = &c_ders[1];
        let c_double_prime = &c_ders[2];

        let c_prime_norm = sqrtf(c_prime[0]*c_prime[0] + c_prime[1]*c_prime[1] + c_prime[2]*c_prime[2]);
        let dot_c_prime_c_double_prime = c_prime[0]*c_double_prime[0] + c_prime[1]*c_double_prime[1] + c_prime[2]*c_double_prime[2];

        u + (v * t) / c_prime_norm - (t*t / 2.0) * (dot_c_prime_c_double_prime / (c_prime_norm * c_prime_norm * c_prime_norm * c_prime_norm)) * v * v
    }
}
