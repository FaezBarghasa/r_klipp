#![no_std]
#![no_main]

use embassy_executor::Executor;
use embassy_time::{Duration, Timer};
use panic_halt as _;

#[embassy_executor::task(pool_size = 1)]
async fn step_generator() {
    loop {
        // Highest priority task
        Timer::after(Duration::from_millis(100)).await;
    }
}

#[embassy_executor::task(pool_size = 1)]
async fn protocol_handler() {
    loop {
        // Medium priority task
        Timer::after(Duration::from_millis(500)).await;
    }
}

#[embassy_executor::task(pool_size = 1)]
async fn safety_monitor() {
    loop {
        // Lowest priority task
        Timer::after(Duration::from_millis(1000)).await;
    }
}

#[embassy_executor::main]
async fn main(spawner: embassy_executor::Spawner) {
    spawner.spawn(step_generator()).unwrap();
    spawner.spawn(protocol_handler()).unwrap();
    spawner.spawn(safety_monitor()).unwrap();
}