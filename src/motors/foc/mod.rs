use micromath::F32Ext;

const SQRT_3_DIV_2: f32 = 0.8660254;

pub fn clarke_transform(a: f32, b: f32, c: f32) -> (f32, f32) {
    let alpha = a;
    let beta = (a + 2.0 * b) / 3.0f32.sqrt();
    (alpha, beta)
}

pub fn park_transform(alpha: f32, beta: f32, angle: f32) -> (f32, f32) {
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    let d = alpha * cos_angle + beta * sin_angle;
    let q = -alpha * sin_angle + beta * cos_angle;
    (d, q)
}

pub fn inv_park_transform(d: f32, q: f32, angle: f32) -> (f32, f32) {
    let cos_angle = angle.cos();
    let sin_angle = angle.sin();
    let alpha = d * cos_angle - q * sin_angle;
    let beta = d * sin_angle + q * cos_angle;
    (alpha, beta)
}

pub fn svpwm(alpha: f32, beta: f32) -> (f32, f32, f32) {
    // Simplified SVPWM implementation
    let x = beta;
    let y = -beta / 2.0 - SQRT_3_DIV_2 * alpha;
    let z = -beta / 2.0 + SQRT_3_DIV_2 * alpha;

    // This is not a complete SVPWM implementation.
    // A full implementation would involve sector detection and timing calculations.
    (x, y, z)
}

pub struct FocController {
    // PID controllers for Id and Iq currents
}

impl FocController {
    // This would run in a hard real-time interrupt
    pub fn update_current_loops(&mut self, d_ref: f32, q_ref: f32, d_meas: f32, q_meas: f32) -> (f32, f32) {
        // Run PID for d and q currents
        (0.0, 0.0) // return voltage commands
    }
}

#[embassy_executor::task]
pub async fn foc_velocity_loop(
    // Add channels for velocity commands and feedback
) {
    loop {
        // Run PID for velocity, which generates q_ref for the current loop
        embassy_time::Timer::after_millis(1).await;
    }
}
