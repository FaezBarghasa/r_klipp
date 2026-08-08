#![no_std]
#![no_main]
#![feature(type_alias_impl_trait)]

use embassy_executor::Executor;
use static_cell::StaticCell;

// Panic handler
#[cfg(not(test))]
use {defmt_rtt as _, panic_probe as _};

mod hal;
mod ipc;
mod tasks;

use hal::traits::{Adc, Pwm};
use ipc::{StepCommandQueue, TelemetryQueue};
use tasks::async_tasks::{planner_task, thermal_task};

// Static memory for the executor and queues
static EXECUTOR: StaticCell<Executor> = StaticCell::new();
static STEP_QUEUE: StaticCell<StepCommandQueue> = StaticCell::new();
static TELEMETRY_QUEUE: StaticCell<TelemetryQueue> = StaticCell::new();

// Mock Hardware for compilation
struct MockAdc;
impl Adc<u16> for MockAdc {
    type Error = ();
    async fn read(&mut self) -> Result<u16, Self::Error> {
        Ok(25) // Simulate 25 degrees C
    }
}

struct MockPwm;
impl Pwm for MockPwm {
    type Error = ();
    async fn set_duty_cycle(&mut self, _duty: u16) -> Result<(), Self::Error> { Ok(()) }
    async fn set_frequency(&mut self, _hz: u32) -> Result<(), Self::Error> { Ok(()) }
    async fn enable(&mut self) -> Result<(), Self::Error> { Ok(()) }
    async fn disable(&mut self) -> Result<(), Self::Error> { Ok(()) }
}


#[cortex_m_rt::entry]
fn main() -> ! {
    // In a real application, hardware would be initialized here.

    let step_queue = STEP_QUEUE.init(StepCommandQueue::new());
    let telemetry_queue = TELEMETRY_QUEUE.init(TelemetryQueue::new());

    let (step_producer, step_consumer) = step_queue.split();
    let (telemetry_producer, telemetry_consumer) = telemetry_queue.split();

    // This is where the hard real-time ISR would be configured
    // and its consumer ends of the queues would be moved.
    // For now, we pass a mock consumer to a dummy function.
    tasks::isr_tasks::initialize_step_generator_isr(step_consumer, telemetry_producer);

    let executor = EXECUTOR.init(Executor::new());

    executor.run(|spawner| {
        spawner.spawn(planner_task(step_producer)).unwrap();
        spawner.spawn(thermal_task(MockAdc, MockPwm)).unwrap();
    });
}
