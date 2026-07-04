use embassy_executor::task;
use crate::math::goertzel::Goertzel;

#[task]
pub async fn resonance_tune_actor(
    // Add motor command and encoder feedback channels
) {
    // 1. Command a frequency sweep (chirp) to the motor
    // 2. Record encoder following error
    // 3. Process error data with Goertzel filter for a range of frequencies
    // 4. Identify the frequency with the highest magnitude (resonant frequency)
    // 5. Send result to configuration manager
    loop {
        embassy_time::Timer::after_secs(10).await;
    }
}

#[task]
pub async fn servo_tune_actor(
    // Add motor command and encoder feedback channels
) {
    // 1. Command a step change in position
    // 2. Record encoder feedback over time
    // 3. Analyze the step response to find overshoot and rise time
    // 4. Calculate PID gains using Ziegler-Nichols or similar method
    // 5. Send gains to configuration manager
    loop {
        embassy_time::Timer::after_secs(10).await;
    }
}
