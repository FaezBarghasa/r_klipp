#![no_std]
#![no_main]

use embassy_executor::InterruptExecutor;
use embassy_time::{Duration, Timer};
use static_cell::StaticCell;
use embassy_stm32::interrupt;
use panic_halt as _;

static EXECUTOR_HIGH: StaticCell<InterruptExecutor> = StaticCell::new();
static EXECUTOR_LOW: StaticCell<InterruptExecutor> = StaticCell::new();

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

#[cortex_m_rt::entry]
fn main() -> ! {
    let p = embassy_stm32::init(Default::default());

    let high_prio_irq = interrupt::take!(EXTI1);
    let low_prio_irq = interrupt::take!(EXTI2);

    let executor_high = EXECUTOR_HIGH.init(InterruptExecutor::new(high_prio_irq));
    let executor_low = EXECUTOR_LOW.init(InterruptExecutor::new(low_prio_irq));

    let spawner_high = executor_high.start();
    let spawner_low = executor_low.start();

    spawner_high.spawn(step_generator()).unwrap();
    spawner_low.spawn(protocol_handler()).unwrap();
    spawner_low.spawn(safety_monitor()).unwrap();

    loop {
        // IDLE
    }
}