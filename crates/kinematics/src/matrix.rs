use core::ops::Mul;

#[derive(Debug, Clone, Copy)]
pub struct Matrix3x3 {
    pub data: [[f32; 3]; 3],
}

impl Matrix3x3 {
    pub fn new(data: [[f32; 3]; 3]) -> Self {
        Self { data }
    }

    pub fn identity() -> Self {
        Self {
            data: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
        }
    }

    pub fn from_euler_angles(a: f32, b: f32, c: f32) -> Self {
        let (sa, ca) = micromath::F32Ext::sin_cos(a.to_radians());
        let (sb, cb) = micromath::F32Ext::sin_cos(b.to_radians());
        let (sc, cc) = micromath::F32Ext::sin_cos(c.to_radians());

        let rz = Matrix3x3::new([
            [cc, -sc, 0.0],
            [sc, cc, 0.0],
            [0.0, 0.0, 1.0],
        ]);

        let ry = Matrix3x3::new([
            [cb, 0.0, sb],
            [0.0, 1.0, 0.0],
            [-sb, 0.0, cb],
        ]);

        let rx = Matrix3x3::new([
            [1.0, 0.0, 0.0],
            [0.0, ca, -sa],
            [0.0, sa, ca],
        ]);

        rz * ry * rx
    }
}

impl Mul for Matrix3x3 {
    type Output = Self;

    fn mul(self, rhs: Self) -> Self::Output {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result[i][j] += self.data[i][k] * rhs.data[k][j];
                }
            }
        }
        Self { data: result }
    }
}

impl Mul<[f32; 3]> for Matrix3x3 {
    type Output = [f32; 3];

    fn mul(self, rhs: [f32; 3]) -> Self::Output {
        let mut result = [0.0; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i] += self.data[i][j] * rhs[j];
            }
        }
        result
    }
}
