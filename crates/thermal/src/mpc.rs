// crates/thermal/src/mpc.rs

pub struct MpcThermalEngine {
    // State Matrix coefficients
    a00: f32, a01: f32,
    a10: f32, a11: f32,
    // Input vector coefficients
    b0: f32, b1: f32,
    // Disturbance coefficients [T_ambient, Volumetric_Flow]
    g00: f32, g01: f32,
    g10: f32, g11: f32,
    
    // Kalman covariance matrices
    p00: f32, p01: f32,
    p10: f32, p11: f32,
    q00: f32, q11: f32, // Process noise diagonals
    r_meas: f32,        // Sensor noise measurement covariance

    // Estimated states
    t_sensor_est: f32,
    t_heater_est: f32,
    
    pub target_temp: f32,
}

impl MpcThermalEngine {
    pub fn new(target_temp: f32) -> Self {
        Self {
            // Empirically tuned discretizations for high-output heater block
            a00: 0.9412, a01: 0.0588,
            a10: 0.0122, a11: 0.9878,
            b0: 0.0011,  b1: 0.0844,
            g00: 0.015,  g01: -0.45,
            g10: 0.005,  g11: -1.25,
            p00: 1.0,    p01: 0.0,
            p10: 0.0,    p11: 1.0,
            q00: 0.001,  q11: 0.005,
            r_meas: 0.01,
            t_sensor_est: 22.0,
            t_heater_est: 22.0,
            target_temp,
        }
    }

    /// Evaluates the hotend thermal step, returning the new power output command (0.0 to 1.0).
    pub fn evaluate(&mut self, y_measured: f32, t_ambient: f32, u_prev: f32, volumetric_flow: f32) -> f32 {
        // 1. Prediction Step: Predict next states based on physical thermal dissipation
        let d0 = t_ambient;
        let d1 = volumetric_flow;
        
        let x_pred_sensor = self.a00 * self.t_sensor_est + self.a01 * self.t_heater_est + self.b0 * u_prev + self.g00 * d0 + self.g01 * d1;
        let x_pred_heater = self.a10 * self.t_sensor_est + self.a11 * self.t_heater_est + self.b1 * u_prev + self.g10 * d0 + self.g11 * d1;

        // Covariance Prediction: P_pred = A_d * P * A_d^T + Q
        let p_pred00 = self.a00 * (self.a00 * self.p00 + self.a01 * self.p10) + self.a01 * (self.a00 * self.p01 + self.a01 * self.p11) + self.q00;
        let p_pred01 = self.a00 * (self.a10 * self.p00 + self.a11 * self.p10) + self.a01 * (self.a10 * self.p01 + self.a11 * self.p11);
        let p_pred10 = self.a10 * (self.a00 * self.p00 + self.a01 * self.p10) + self.a11 * (self.a00 * self.p01 + self.a01 * self.p11);
        let p_pred11 = self.a10 * (self.a10 * self.p00 + self.a11 * self.p10) + self.a11 * (self.a10 * self.p01 + self.a11 * self.p11) + self.q11;

        // 2. Correction Step (Innovation Kalman updates)
        let innovation = y_measured - x_pred_sensor;
        let s = p_pred00 + self.r_meas;
        let k0 = p_pred00 / s;
        let k1 = p_pred10 / s;

        // Adjust state estimates with new measurement weightings
        self.t_sensor_est = x_pred_sensor + k0 * innovation;
        self.t_heater_est = x_pred_heater + k1 * innovation;

        // Update error covariance matrices
        self.p00 = (1.0 - k0) * p_pred00;
        self.p01 = (1.0 - k0) * p_pred01;
        self.p10 = p_pred10 - k1 * p_pred00;
        self.p11 = p_pred11 - k1 * p_pred01;

        // 3. Controller Rule: Calculate next PWM command using feed-forward plus estimated internal error
        let heater_error = self.target_temp - self.t_heater_est;
        let feedback_power = heater_error * 0.075;
        let feedforward_loss = (self.t_heater_est - t_ambient) * 0.0022 + volumetric_flow * 0.065;
        
        (feedback_power + feedforward_loss).clamp(0.0, 1.0)
    }
}
