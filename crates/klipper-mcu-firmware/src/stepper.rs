//! # Stepper Motor Control
//!
//! This module is responsible for controlling the 3D printer's stepper motors. It uses a
//! hardware timer to generate step pulses with precise timing, ensuring smooth and
//! accurate movement.

use stm32_metapac as pac;
use stm32_metapac::interrupt;
use embassy_stm32::interrupt::{Interrupt, InterruptExt};
use mcu_drivers::stepper::StepperController;

pub const STEPPER_QUEUE_SIZE: usize = 256;

// Statically allocate the stepper controller with 256 segments capacity.
// PE2 step pin (bit 2), PB8 direction pin (bit 8)
pub static mut STEPPER_CONTROLLER: StepperController<STEPPER_QUEUE_SIZE> = StepperController::new(1 << 2, 1 << 8);

// Keep track of the current direction state
static mut CURRENT_DIR: bool = false;

/// Configures TIM2 as an 84MHz 32-bit hardware timer, enables its update interrupt,
/// and registers the interrupt handler.
pub fn init_stepper_timer() {
    unsafe {
        // 1. Enable TIM2 clock in RCC (APB1ENR)
        let mut apb1 = pac::RCC.apb1enr().read();
        apb1.set_tim2en(true);
        pac::RCC.apb1enr().write_value(apb1);

        // 2. Configure TIM2
        // Prescaler: 0 (ticks at 84MHz)
        pac::TIM2.psc().write_value(pac::timer::regs::Psc(0));

        // Auto-Reload Register: initial value (large enough)
        pac::TIM2.arr().write_value(pac::timer::regs::Arr32(84000));

        // Clear update interrupt flag
        let mut sr = pac::TIM2.sr().read();
        sr.set_uif(false);
        pac::TIM2.sr().write_value(sr);

        // Enable update interrupt (UIE)
        let mut dier = pac::TIM2.dier().read();
        dier.set_uie(true);
        pac::TIM2.dier().write_value(dier);

        // 3. Configure NVIC for TIM2 using compile-time interrupt handler registration
        Interrupt::TIM2.set_priority(embassy_stm32::interrupt::Priority::P1); // High priority
        Interrupt::TIM2.enable();

        // 4. Start TIM2
        let mut cr1 = pac::TIM2.cr1().read();
        cr1.set_cen(true);
        pac::TIM2.cr1().write_value(cr1);
        
        defmt::info!("TIM2 Stepper timer initialized at 84MHz");
    }
}

/// TIM2 Interrupt Handler
#[cortex_m_rt::interrupt]
unsafe fn TIM2() {
    // Clear the update interrupt flag (UIF)
    let mut sr = pac::TIM2.sr().read();
    sr.set_uif(false);
    pac::TIM2.sr().write_value(sr);

    // Dequeue next step segment if available
    if let Some(segment) = STEPPER_CONTROLLER.dequeue_segment() {
        // GPIOB BSRR for direction pin (PB8)
        let gpiob_bsrr = 0x4002_0418 as *mut u32;
        // GPIOE BSRR for step pin (PE2)
        let gpioe_bsrr = 0x4002_1018 as *mut u32;
        // TIM2 ARR register to set next step interval
        let tim2_arr = 0x4000_002C as *mut u32;

        // Set direction pin if changed
        if CURRENT_DIR != segment.direction {
            CURRENT_DIR = segment.direction;
            if CURRENT_DIR {
                core::ptr::write_volatile(gpiob_bsrr, 1 << 8); // Set PB8 high
            } else {
                core::ptr::write_volatile(gpiob_bsrr, 1 << 24); // Set PB8 low (reset)
            }
        }

        // Write interval_ticks directly to the TIM2 ARR register
        core::ptr::write_volatile(tim2_arr, segment.interval_ticks);

        // Generate step pulse: toggle PE2 (bit 2)
        core::ptr::write_volatile(gpioe_bsrr, 1 << 2);      // Set PE2 high
        core::ptr::write_volatile(gpioe_bsrr, 1 << 18);     // Set PE2 low (reset)
    }
}

/// The stepper control task.
///
/// This task manages the stepper motor queue and generates step pulses.
#[embassy_executor::task]
pub async fn stepper_task() {
    defmt::info!("Stepper task started");

    // Initialize timer and interrupts
    init_stepper_timer();

    loop {
        // Stepper task stays active, processing G-code queued moves.
        // In a complete implementation, this task parses protocol messages and enqueues StepSegments.
        embassy_time::Timer::after(embassy_time::Duration::from_secs(10)).await;
    }
}