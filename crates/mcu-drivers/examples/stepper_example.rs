#![no_std]
#![no_main]

use defmt_rtt as _;
use mcu_drivers::stepper::{StepSegment, StepperController};
use panic_probe as _;

#[cortex_m_rt::entry]
fn main() -> ! {
    defmt::info!("--- Stepper Control Example ---");

    let mut controller = StepperController::<256>::new(0b0001, 0b0010);

    for i in 0..5 {
        let segment = StepSegment {
            interval_ticks: 1000 + (i * 100),
            direction: true,
            enable_mask: 1 << 0,
        };
        controller.enqueue_segment(segment).expect("Failed to enqueue segment");
    }

    let mut bsrr_dummy: u32 = 0;
    let mut arr_dummy: u32 = 0;

    let mut steps_processed = 0;
    while let Some(_) = controller.dequeue_segment() {
        steps_processed += 1;
        unsafe {
            controller.execute_next_step_isr(&mut bsrr_dummy as *mut u32, &mut arr_dummy as *mut u32);
        }
    }

    loop {
        cortex_m::asm::wfi();
    }
}
