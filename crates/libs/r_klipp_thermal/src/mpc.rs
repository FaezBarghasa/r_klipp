
// Kalman Filter implementation for temperature estimation
pub struct KalmanFilter {
    q: f32, // process noise
    r: f32, // measurement noise
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

// Simplified MPC for thermal control
pub struct ThermalMpc;

impl ThermalMpc {
    pub fn new() -> Self {
        Self
    }

    pub fn calculate_pwm(&self, current_temp: f32, target_temp: f32) -> f32 {
        let error = target_temp - current_temp;
        // A real MPC would have a model of the system and predict future states.
        // This is a simplified proportional controller for now.
        (error * 0.1).max(0.0).min(1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kalman_filter() {
        let mut kf = KalmanFilter::new(0.01, 0.1);
        let measurements = [25.1, 25.0, 24.9, 25.2, 24.8];
        let mut estimated_temp = 0.0;
        for m in measurements {
            estimated_temp = kf.update(m);
        }
        // Assert that the estimated temperature is close to the average
        assert!((estimated_temp - 25.0).abs() < 0.1);
    }
}
