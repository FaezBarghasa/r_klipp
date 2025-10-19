// File: crates/motion/benches/real_time_perf.rs
use criterion::{criterion_group, criterion_main, Criterion, Throughput, BenchmarkId};
use mcu_drivers::stepper::{StepCommand, StepperController, AtomicGpioPort, Timer};
use heapless::spsc::Queue;
use core::cell::RefCell;
use critical_section::Mutex;

// --- Mock Hardware for Benchmarking ---
struct MockPort;
impl AtomicGpioPort for MockPort {
    fn set_and_clear_atomic(&mut self, _set_mask: u8, _clear_mask: u8) {}
    fn write(&mut self, _mask: u8) {}
}
struct MockTimer;
impl Timer for MockTimer {
    fn schedule_next(&mut self, _ticks: u16) {}
    fn trigger_now(&mut self) {}
    fn stop(&mut self) {}
}

static STEP_PORT: Mutex<RefCell<MockPort>> = Mutex::new(RefCell::new(MockPort));
static DIR_PORT: Mutex<RefCell<MockPort>> = Mutex::new(RefCell::new(MockPort));
static TIMER: Mutex<RefCell<MockTimer>> = Mutex::new(RefCell::new(MockTimer));
static mut COMMAND_QUEUE: Queue<StepCommand, 256> = Queue::new();

/// Benchmark for the worst-case execution time (WCET) of the stepper ISR.
fn bench_stepper_isr_wcet(c: &mut Criterion) {
    let (_producer, consumer) = unsafe { COMMAND_QUEUE.split() };
    let mut controller = StepperController::<8>::new(consumer);

    // Pre-fill the controller's internal state to simulate a real scenario
    controller.start(&DIR_PORT, &TIMER);

    c.bench_function("stepper_isr_wcet", |b| {
        b.iter(|| {
            // This is the function that would be called inside the timer interrupt.
            // We are measuring its execution time to estimate real-time performance.
            controller.on_timer_interrupt(&STEP_PORT, &DIR_PORT, &TIMER);
        })
    });
}

criterion_group!(benches, bench_stepper_isr_wcet);
criterion_main!(benches);

