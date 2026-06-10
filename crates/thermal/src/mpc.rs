pub struct MpcThermalEngine {
    pub a00: f32, pub a01: f32,
    pub a10: f32, pub a11: f32,
    pub b0: f32, pub b1: f32,
    pub g00: f32, pub g01: f32,
    pub g10: f32, pub g11: f32,
    
    pub p00: f32, pub p01: f32,
    pub p10: f32, pub p11: f32,
    pub q00: f32, pub q11: f32,
    pub r_meas: f32,
    
    pub x0: f32,
    pub x1: f32,
    
    pub target_temp: f32,
    pub k_p: f32,
    pub ff_loss: f32,
}

impl MpcThermalEngine {
    pub fn step(&mut self, y_measured: f32, t_ambient: f32, u_prev: f32, volumetric_flow: f32) -> f32 {
        let d0 = t_ambient;
        let d1 = volumetric_flow;

        let x0_pred = self.a00 * self.x0 + self.a01 * self.x1 + self.b0 * u_prev + self.g00 * d0 + self.g01 * d1;
        let x1_pred = self.a10 * self.x0 + self.a11 * self.x1 + self.b1 * u_prev + self.g10 * d0 + self.g11 * d1;

        let p00_temp = self.a00 * self.p00 + self.a01 * self.p10;
        let p01_temp = self.a00 * self.p01 + self.a01 * self.p11;
        let p10_temp = self.a10 * self.p00 + self.a11 * self.p10;
        let p11_temp = self.a10 * self.p01 + self.a11 * self.p11;

        let p00_pred = p00_temp * self.a00 + p01_temp * self.a01 + self.q00;
        let p01_pred = p00_temp * self.a10 + p01_temp * self.a11;
        let p10_pred = p10_temp * self.a00 + p11_temp * self.a01;
        let p11_pred = p10_temp * self.a10 + p11_temp * self.a11 + self.q11;

        let s = p00_pred + self.r_meas;
        let s_inv = if s != 0.0 { 1.0 / s } else { 0.0 };
        
        let k0 = p00_pred * s_inv;
        let k1 = p10_pred * s_inv;

        let y_err = y_measured - x0_pred;
        self.x0 = x0_pred + k0 * y_err;
        self.x1 = x1_pred + k1 * y_err;

        let i00 = 1.0 - k0;
        let i01 = 0.0;
        let i10 = -k1;
        let i11 = 1.0;

        self.p00 = i00 * p00_pred + i01 * p10_pred;
        self.p01 = i00 * p01_pred + i01 * p11_pred;
        self.p10 = i10 * p00_pred + i11 * p10_pred;
        self.p11 = i10 * p01_pred + i11 * p11_pred;

        let mut u_k = self.k_p * (self.target_temp - self.x1) + self.ff_loss;
        if u_k < 0.0 {
            u_k = 0.0;
        } else if u_k > 1.0 {
            u_k = 1.0;
        }
        
        u_k
    }
}