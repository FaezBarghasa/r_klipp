//! Kalman-based disturbance observer.

pub struct DisturbanceObserver {
    // State: [position, velocity, disturbance_force]
    x: [f32; 3],
    // Covariance matrix
    p: [[f32; 3]; 3],
    // Process noise
    q: [[f32; 3]; 3],
    // Measurement noise
    r: f32,
    // System model
    a: [[f32; 3]; 3],
    b: [f32; 3],
    h: [f32; 3],
}

impl DisturbanceObserver {
    pub fn new(dt: f32) -> Self {
        Self {
            x: [0.0; 3],
            p: [[1.0, 0.0, 0.0], [0.0, 1.0, 0.0], [0.0, 0.0, 1.0]],
            q: [[0.01, 0.0, 0.0], [0.0, 0.01, 0.0], [0.0, 0.0, 0.01]],
            r: 0.1,
            a: [
                [1.0, dt, 0.0],
                [0.0, 1.0, dt],
                [0.0, 0.0, 1.0],
            ],
            b: [0.5 * dt * dt, dt, 0.0],
            h: [1.0, 0.0, 0.0],
        }
    }

    pub fn update(&mut self, u: f32, z: f32) {
        // Predict
        let x_pred = [
            self.a[0][0]*self.x[0] + self.a[0][1]*self.x[1] + self.a[0][2]*self.x[2] + self.b[0]*u,
            self.a[1][0]*self.x[0] + self.a[1][1]*self.x[1] + self.a[1][2]*self.x[2] + self.b[1]*u,
            self.a[2][0]*self.x[0] + self.a[2][1]*self.x[1] + self.a[2][2]*self.x[2] + self.b[2]*u,
        ];

        let ap = self.mat_mul_3x3(&self.a, &self.p);
        let a_t = self.transpose_3x3(&self.a);
        let p_pred = self.mat_add_3x3(&self.mat_mul_3x3(&ap, &a_t), &self.q);

        // Update
        let y = z - (self.h[0]*x_pred[0] + self.h[1]*x_pred[1] + self.h[2]*x_pred[2]);
        let s = self.h[0]*p_pred[0][0]*self.h[0] + self.h[1]*p_pred[1][1]*self.h[1] + self.h[2]*p_pred[2][2] + self.r;
        let k = [
            p_pred[0][0]*self.h[0]/s,
            p_pred[1][0]*self.h[0]/s,
            p_pred[2][0]*self.h[0]/s,
        ];

        self.x[0] = x_pred[0] + k[0]*y;
        self.x[1] = x_pred[1] + k[1]*y;
        self.x[2] = x_pred[2] + k[2]*y;

        let kh = [
            [k[0]*self.h[0], k[0]*self.h[1], k[0]*self.h[2]],
            [k[1]*self.h[0], k[1]*self.h[1], k[1]*self.h[2]],
            [k[2]*self.h[0], k[2]*self.h[1], k[2]*self.h[2]],
        ];
        let ikh = self.mat_sub_3x3(&[[1.0,0.0,0.0],[0.0,1.0,0.0],[0.0,0.0,1.0]], &kh);
        self.p = self.mat_mul_3x3(&ikh, &p_pred);
    }

    pub fn get_disturbance(&self) -> f32 {
        self.x[2]
    }

    fn mat_mul_3x3(&self, a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
        let mut res = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    res[i][j] += a[i][k] * b[k][j];
                }
            }
        }
        res
    }

    fn mat_add_3x3(&self, a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
        let mut res = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                res[i][j] = a[i][j] + b[i][j];
            }
        }
        res
    }

    fn mat_sub_3x3(&self, a: &[[f32; 3]; 3], b: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
        let mut res = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                res[i][j] = a[i][j] - b[i][j];
            }
        }
        res
    }

    fn transpose_3x3(&self, m: &[[f32; 3]; 3]) -> [[f32; 3]; 3] {
        let mut res = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                res[i][j] = m[j][i];
            }
        }
        res
    }
}
