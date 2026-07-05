use micromath::F32Ext;

// Simple Kalman filter implementation
pub struct KalmanFilter {
    q: f32, // process noise covariance
    r: f32, // measurement noise covariance
    x: f32, // value
    p: f32, // estimation error covariance
    k: f32, // kalman gain
}

impl KalmanFilter {
    pub fn new(q: f32, r: f32) -> Self {
        Self { q, r, x: 0.0, p: 0.0, k: 0.0 }
    }

    pub fn update(&mut self, measurement: f32) -> f32 {
        self.p += self.q;
        self.k = self.p / (self.p + self.r);
        self.x += self.k * (measurement - self.x);
        self.p *= 1.0 - self.k;
        self.x
    }
}

// Simplified Model Predictive Control
pub struct ThermalMpc {
    kalman: KalmanFilter,
    target_temp: f32,
}

impl ThermalMpc {
    pub fn new() -> Self {
        Self {
            kalman: KalmanFilter::new(0.01, 0.1),
            target_temp: 0.0,
        }
    }

    pub fn set_target(&mut self, temp: f32) {
        self.target_temp = temp;
    }

    pub fn update(&mut self, measurement: f32) -> u8 {
        let estimated_temp = self.kalman.update(measurement);
        let error = self.target_temp - estimated_temp;

        // This is a very simplified MPC, a real one would have a more complex model
        let pwm = (error * 10.0).max(0.0).min(255.0) as u8;
        pwm
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kalman_filter() {
        let mut kalman = KalmanFilter::new(0.01, 0.1);
        let true_value = 200.0;
        let measurements = [199.5, 200.1, 200.3, 199.8, 200.0];
        let mut estimated_value = 0.0;
        for m in measurements {
            estimated_value = kalman.update(m);
        }
        assert!((estimated_value - true_value).abs() < 0.1);
    }
}
