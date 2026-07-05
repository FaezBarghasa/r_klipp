use embassy_executor::task;
use heapless::spsc::{Receiver, Sender};

use crate::hal::traits::{Encoder, Motor};
use crate::math::goertzel::Goertzel;

const RESONANCE_TEST_FREQ_START: f32 = 20.0;
const RESONANCE_TEST_FREQ_END: f32 = 150.0;
const RESONANCE_TEST_FREQ_STEP: f32 = 1.0;
const RESONANCE_TEST_AMPLITUDE: f32 = 100.0; // In encoder counts
const SAMPLE_RATE_HZ: f32 = 1000.0;

#[task]
pub async fn resonance_tune_actor(
    mut motor: impl Motor,
    mut encoder: impl Encoder,
    mut result_sender: Sender<'static, f32, 1>,
) {
    let mut max_magnitude = 0.0;
    let mut resonant_freq = 0.0;

    let mut freq = RESONANCE_TEST_FREQ_START;
    while freq <= RESONANCE_TEST_FREQ_END {
        let mut goertzel = Goertzel::new(freq, SAMPLE_RATE_HZ);
        let mut last_pos_cmd = 0.0;

        for i in 0..1024 {
            let time = i as f32 / SAMPLE_RATE_HZ;
            let pos_cmd = RESONANCE_TEST_AMPLITUDE * libm::sin(2.0 * core::f32::consts::PI * freq * time);
            let vel_cmd = (pos_cmd - last_pos_cmd) * SAMPLE_RATE_HZ;
            last_pos_cmd = pos_cmd;

            motor.set_velocity(vel_cmd).await.ok();
            let measured_pos = encoder.get_position().await.unwrap_or(0) as f32;
            let error = pos_cmd - measured_pos;
            goertzel.process_sample(error);

            embassy_time::Timer::after_micros(1000).await;
        }

        let magnitude = goertzel.get_magnitude();
        if magnitude > max_magnitude {
            max_magnitude = magnitude;
            resonant_freq = freq;
        }

        freq += RESONANCE_TEST_FREQ_STEP;
    }

    result_sender.enqueue(resonant_freq).ok();
}

#[task]
pub async fn servo_tune_actor(
    mut motor: impl Motor,
    mut encoder: impl Encoder,
    mut result_sender: Sender<'static, (f32, f32, f32), 1>,
) {
    let step_amplitude = 1000.0; // counts
    motor.set_position(step_amplitude).await.ok();

    let start_time = embassy_time::Instant::now();
    let mut max_overshoot = 0.0;
    let mut rise_time = 0.0;
    let mut peak_time = 0.0;

    loop {
        let now = start_time.elapsed().as_secs_f32();
        let pos = encoder.get_position().await.unwrap_or(0) as f32;

        if pos > max_overshoot {
            max_overshoot = pos;
            peak_time = now;
        }

        if rise_time == 0.0 && pos >= step_amplitude * 0.9 {
            rise_time = now;
        }

        if now > 2.0 { // 2 second timeout
            break;
        }
        embassy_time::Timer::after_micros(1000).await;
    }

    let overshoot_ratio = (max_overshoot - step_amplitude) / step_amplitude;

    // A simple tuning rule based on overshoot
    let kp = if overshoot_ratio > 0.5 { 0.3 } else { 0.7 };
    let ki = kp / (2.0 * rise_time);
    let kd = kp * peak_time / 8.0;

    result_sender.enqueue((kp, ki, kd)).ok();
}