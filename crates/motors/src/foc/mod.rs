use micromath::F32Ext;

const SQRT_3: f32 = 1.7320508;

pub fn clarke_transform(a: f32, b: f32, _c: f32) -> (f32, f32) {
    let alpha = a;
    let beta = (a + 2.0 * b) / SQRT_3;
    (alpha, beta)
}

pub fn park_transform(alpha: f32, beta: f32, angle: f32) -> (f32, f32) {
    let (sin, cos) = angle.sin_cos();
    let d = alpha * cos + beta * sin;
    let q = -alpha * sin + beta * cos;
    (d, q)
}

pub fn inverse_park_transform(d: f32, q: f32, angle: f32) -> (f32, f32) {
    let (sin, cos) = angle.sin_cos();
    let alpha = d * cos - q * sin;
    let beta = d * sin + q * cos;
    (alpha, beta)
}

pub fn svpwm(alpha: f32, beta: f32) -> (f32, f32, f32) {
    // Simplified SVPWM for now. A real implementation would involve sector detection.
    let t1 = alpha;
    let t2 = beta;
    let t3 = 1.0 - t1 - t2;
    (t1, t2, t3)
}

pub struct FocController {
    // PID controllers for Id and Iq would go here
}

impl FocController {
    pub fn new() -> Self {
        Self {}
    }

    pub fn step(&mut self, i_a: f32, i_b: f32, i_c: f32, angle: f32, target_iq: f32) -> (f32, f32, f32) {
        let (alpha, beta) = clarke_transform(i_a, i_b, i_c);
        let (_d, q) = park_transform(alpha, beta, angle);

        let error = target_iq - q;
        let voltage_q = error * 0.1; // Simplified P controller

        let (v_alpha, v_beta) = inverse_park_transform(0.0, voltage_q, angle);
        svpwm(v_alpha, v_beta)
    }
}

#[embassy_executor::task]
pub async fn foc_controller_task(mut controller: FocController) {
    loop {
        // This task would run the velocity and position loops.
        embassy_time::Timer::after(embassy_time::Duration::from_millis(1)).await;
    }
}
