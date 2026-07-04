//! ISR-based "hard real-time" tasks for r_klipp.
//! This module contains the interrupt handlers that must execute with very
//! low and predictable latency.
//! This file corresponds to Task 1.4 of the development plan.

#![no_std]

use crate::ipc::{StepCommand, TelemetryPacket, StepCommandQueue, TelemetryQueue};
use crate::hal::{Gpio, InterruptMasker};
use heapless::spsc::{Consumer, Producer};
use cortex_m::interrupt;

// These would be provided by the hardware-specific HAL.
// We define them here as placeholders.
struct TimerIsr;
impl TimerIsr {
    fn clear_update_interrupt_flag(&mut self) { /* no-op */ }
    fn set_auto_reload_value(&mut self, _value: u32) { /* no-op */ }
}

struct StepPins;
impl Gpio for StepPins {
    type Error = ();
    fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn is_high(&self) -> Result<bool, Self::Error> { Ok(false) }
    fn is_low(&self) -> Result<bool, Self::Error> { Ok(true) }
}

struct DirPins;
impl Gpio for DirPins {
    type Error = ();
    fn set_high(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn set_low(&mut self) -> Result<(), Self::Error> { Ok(()) }
    fn is_high(&self) -> Result<bool, Self::Error> { Ok(false) }
    fn is_low(&self) -> Result<bool, Self::Error> { Ok(true) }
}


/// The Step Generator ISR. This is the heart of the motion system.
/// It is responsible for generating the step pulses for the motors with
/// precise timing.
///
/// # Safety
///
/// This function is intended to be called from an interrupt handler.
/// It must not block, and its execution time must be minimal and predictable.
/// The `STEP_COMMAND_CONSUMER` and `TELEMETRY_PRODUCER` must be `Some` before
/// this ISR is enabled.
pub unsafe fn step_generator_isr(
    step_consumer: &mut Consumer<'static, StepCommand, 1024>,
    telemetry_producer: &mut Producer<'static, TelemetryPacket, 1024>,
    timer: &mut TimerIsr,
    step_pins: &mut StepPins,
    dir_pins: &mut DirPins,
    interrupt_masker: &impl InterruptMasker,
) {
    // Clear the timer's update interrupt flag to acknowledge the interrupt.
    timer.clear_update_interrupt_flag();

    // Critical section: We must not be interrupted while accessing the queue.
    interrupt_masker.disable_interrupts();

    if let Some(cmd) = step_consumer.dequeue() {
        // Re-enable interrupts as soon as we are done with the queue.
        interrupt_masker.enable_interrupts();

        // 1. Set the direction pins based on the command's direction mask.
        //    This is a simplified example. A real implementation would iterate
        //    through the bits of the mask and set each direction pin accordingly.
        if (cmd.direction_mask & 1) != 0 {
            dir_pins.set_high().ok();
        } else {
            dir_pins.set_low().ok();
        }

        // 2. Generate the step pulse.
        //    This is a simplified example. A real implementation would likely
        //    involve a very short delay or a separate timer for the pulse width.
        step_pins.set_high().ok();
        // A short delay would be here in a real system.
        step_pins.set_low().ok();

        // 3. Set the timer for the next step.
        timer.set_auto_reload_value(cmd.interval);

        // 4. Send telemetry back to the async world.
        let packet = TelemetryPacket {
            timestamp: 0, // In a real system, we would get this from a high-res timer.
            completed_mask: cmd.step_mask,
        };
        // This enqueue can fail if the queue is full. In a hard real-time
        // system, this is a design error. We ignore the error here.
        let _ = telemetry_producer.enqueue(packet);

    } else {
        // No command in the queue. Re-enable interrupts.
        interrupt_masker.enable_interrupts();
        // We could disable the timer here to save power, and the async task
        // would re-enable it when it enqueues the next command.
    }
}
