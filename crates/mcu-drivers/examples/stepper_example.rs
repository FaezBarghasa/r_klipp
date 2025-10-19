#![no_std]
#![no_main]

use core::cell::RefCell;
use critical_section::Mutex;
use defmt_rtt as _;
use heapless::spsc::Queue;
use mcu_drivers::stepper::{GpioPort, StepCommand, StepperController, Timer};
use panic_probe as _;

// Mock hardware for the example
struct MockPort(pub &'static str);
impl GpioPort for MockPort {
    fn set(&mut self, mask: u8) {
        defmt::info!("{}: SET pins {:08b}", self.0, mask);
    }
    fn clear(&mut self, mask: u8) {
        defmt::info!("{}: CLEAR pins {:08b}", self.0, mask);
    }
    fn write(&mut self, mask: u8) {
        defmt::info!("{}: WRITE pins {:08b}", self.0, mask);
    }
}

struct MockTimer;
impl Timer for MockTimer {
    fn schedule_next(&mut self, ticks: u16) {
        defmt::info!("TIMER: Scheduled next interrupt in {} ticks.", ticks);
    }
    fn trigger_now(&mut self) {
        defmt::info!("TIMER: Triggering next interrupt now.");
    }
    fn stop(&mut self) {
        defmt::info!("TIMER: Stopped.");
    }
}

// Static resources needed for the firmware
static STEP_PORT: Mutex<RefCell<MockPort>> = Mutex::new(RefCell::new(MockPort("STEP")));
static DIR_PORT: Mutex<RefCell<MockPort>> = Mutex::new(RefCell::new(MockPort("DIR")));
static TIMER: Mutex<RefCell<MockTimer>> = Mutex::new(RefCell::new(MockTimer));
static mut COMMAND_QUEUE: Queue<StepCommand, 256> = Queue::new();

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("--- Stepper Control Example ---");

    let (mut producer, consumer) = unsafe { COMMAND_QUEUE.split() };
    let mut controller = StepperController::<4>::new(consumer);

    defmt::info!("Enqueuing a sequence of 5 steps for motor 0...");
    for i in 0..5 {
        let cmd = StepCommand {
            stepper_mask: 1 << 0, // Motor 0
            direction_mask: 1 << 0, // Forward
            interval_ticks: 1000 + (i * 100),
        };
        producer.enqueue(cmd).expect("Failed to enqueue command");
    }
    defmt::info!("Enqueuing a sequence of 3 steps for motor 1 (backwards)...");
    for _ in 0..3 {
        let cmd = StepCommand {
            stepper_mask: 1 << 1, // Motor 1
            direction_mask: 0,    // Backward
            interval_ticks: 800,
        };
        producer.enqueue(cmd).expect("Failed to enqueue command");
    }

    defmt::info!("\n--- Simulating Timer Interrupts ---");
    // In a real application, a hardware interrupt would call `process_next_step`.
    // Here, we simulate it by calling it in a loop until the queue is empty.
    let mut steps_processed = 0;
    while producer.len() > 0 {
        steps_processed +=1;
        defmt::info!("\n--- Interrupt Fired (Step {}) ---", steps_processed);
        controller.process_next_step(&STEP_PORT, &DIR_PORT, &TIMER);
    }

    defmt::info!("\n--- Interrupt Fired (Queue should be empty) ---");
    controller.process_next_step(&STEP_PORT, &DIR_PORT, &TIMER);


    defmt::info!("\n--- Final Stepper Positions ---");
    defmt::info!("Motor 0: {}", controller.get_position(0).unwrap_or_default());
    defmt::info!("Motor 1: {}", controller.get_position(1).unwrap_or_default());
    defmt::info!("Motor 2: {}", controller.get_position(2).unwrap_or_default());

    loop {
        cortex_m::asm::wfi();
    }
}
