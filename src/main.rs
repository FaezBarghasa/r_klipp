//! Main entry point for the r_klipp firmware.
//! This file initializes the hardware, sets up the async executor,
//! and spawns the initial tasks.
//! This file corresponds to Task 1.3 of the development plan.

#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Executor;
use embassy_time::{Timer, Duration};
use static_cell::StaticCell;

// Panic handler
#[cfg(not(test))]
use {defmt_rtt as _, panic_probe as _};

mod hal;
mod ipc;
mod tasks;

use hal::traits::{Adc, Pwm};
use ipc::{StepCommandQueue, TelemetryQueue, GCodeAstNode};
use tasks::async_tasks::{planner_task, thermal_task};

// Static memory for the executor and queues
static EXECUTOR: StaticCell<Executor> = StaticCell::new();
static STEP_QUEUE: StaticCell<StepCommandQueue<1024>> = StaticCell::new();
static GCODE_QUEUE: StaticCell<ipc::GCodeQueue<64>> = StaticCell::new();
static TELEMETRY_QUEUE: StaticCell<TelemetryQueue<1024>> = StaticCell::new();

// --- Mock Hardware for Compilation ---
// In a real application, these would be initialized from a hardware-specific
// HAL crate (e.g., `stm32f4xx_hal`).

struct MockAdc;
impl Adc<u16> for MockAdc {
    type Error = ();
    async fn read(&mut self) -> Result<u16, Self::Error> {
        // Return a dummy value
        Ok(1500)
    }
}

struct MockPwm;
impl Pwm for MockPwm {
    fn set_duty_cycle(&mut self, _duty: u16) {
        // No-op
    }
    fn get_max_duty(&self) -> u16 {
        // Return a dummy value
        4095
    }
}


#[cortex_m_rt::entry]
fn main() -> ! {
    // 1. Initialize hardware peripherals (clocks, GPIO, ADC, PWM, etc.).
    //    This is highly target-specific and is omitted here.

    // 2. Initialize the static queues.
    let step_queue = STEP_QUEUE.init(StepCommandQueue::new());
    let gcode_queue = GCODE_QUEUE.init(ipc::GCodeQueue::new());
    let telemetry_queue = TELEMETRY_QUEUE.init(TelemetryQueue::new());

    // 3. Split the queues into their producer and consumer ends.
    let (step_producer, _step_consumer) = step_queue.split();
    let (_gcode_producer, gcode_consumer) = gcode_queue.split();
    let (_telemetry_producer, _telemetry_consumer) = telemetry_queue.split();

    // 4. Initialize the executor.
    let executor = EXECUTOR.init(Executor::new());

    // 5. Spawn the async tasks.
    executor.run(|spawner| {
        spawner.spawn(planner_task(step_producer, gcode_consumer)).unwrap();
        spawner.spawn(thermal_task(MockAdc, MockPwm)).unwrap();
    });

    // The executor's `run` method never returns.
    // If it did, we would loop forever.
    loop {
        cortex_m::asm::wfi(); // Wait for interrupt
    }
}

// Add a GCodeQueue to the ipc module for compilation
mod ipc {
    pub use super::ipc::*;
    use heapless::spsc::Queue;
    pub type GCodeQueue<const N: usize> = Queue<GCodeAstNode, N>;
}
