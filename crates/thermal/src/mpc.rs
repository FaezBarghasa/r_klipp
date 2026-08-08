use crate::safety::{RunawayWatchdog, RunawayWatchdogConfig};

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
    
    pub x0: f32, // Estimated temperature
    pub x1: f32, // Estimated temperature rate or other state
    
    pub target_temp: f32,
    pub k_p: f32,
    pub ff_loss: f32,

    runaway_watchdog: RunawayWatchdog,
    last_u_k: f32, // Store the last control output (heater power)
}

impl MpcThermalEngine {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        a_coeffs: (f32, f32, f32, f32), // a00, a01, a10, a11
        b_coeffs: (f32, f32),           // b0, b1
        g_coeffs: (f32, f32, f32, f32), // g00, g01, g10, g11
        p_coeffs: (f32, f32, f32, f32), // p00, p01, p10, p11
        q_coeffs: (f32, f32),           // q00, q11
        r_meas: f32,
        initial_x0: f32,
        initial_x1: f32,
        target_temp: f32,
        k_p: f32,
        ff_loss: f32,
        watchdog_config: RunawayWatchdogConfig,
        initial_timestamp_ms: u32,
    ) -> Self {
        Self {
            a00: a_coeffs.0, a01: a_coeffs.1,
            a10: a_coeffs.2, a11: a_coeffs.3,
            b0: b_coeffs.0, b1: b_coeffs.1,
            g00: g_coeffs.0, g01: g_coeffs.1,
            g10: g_coeffs.2, g11: g_coeffs.3,
            p00: p_coeffs.0, p01: p_coeffs.1,
            p10: p_coeffs.2, p11: p_coeffs.3,
            q00: q_coeffs.0, q11: q_coeffs.1,
            r_meas,
            x0: initial_x0,
            x1: initial_x1,
            target_temp,
            k_p,
            ff_loss,
            runaway_watchdog: RunawayWatchdog::new(watchdog_config, initial_x0, initial_timestamp_ms),
            last_u_k: 0.0, // Initialize last heater power to 0
        }
    }

    pub fn step(&mut self, y_measured: f32, t_ambient: f32, volumetric_flow: f32, current_timestamp_ms: u32) -> f32 {
        // Check for E-Stop before performing control logic
        if self.runaway_watchdog.is_e_stop_triggered() {
            return 0.0; // Heater off if E-Stop is active
        }

        let d0 = t_ambient;
        let d1 = volumetric_flow;

        let x0_pred = self.a00 * self.x0 + self.a01 * self.x1 + self.b0 * self.last_u_k + self.g00 * d0 + self.g01 * d1;
        let x1_pred = self.a10 * self.x0 + self.a11 * self.x1 + self.b1 * self.last_u_k + self.g10 * d0 + self.g11 * d1;

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

        let mut u_k = self.k_p * (self.target_temp - self.x0) + self.ff_loss; // Use x0 (estimated temp) for control
        if u_k < 0.0 {
            u_k = 0.0;
        } else if u_k > 1.0 {
            u_k = 1.0;
        }
        
        // Update last_u_k for the next iteration
        self.last_u_k = u_k;

        // Check for thermal runaway with the estimated temperature and current heater power
        let heater_active = u_k > 0.0;
        if self.runaway_watchdog.check_temperature(self.x0, current_timestamp_ms, heater_active) {
            return 0.0; // Trigger E-Stop, turn heater off
        }

        u_k
    }
}