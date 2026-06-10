#![no_std]

use heapless::spsc::Queue;

/// Represents a single movement segment for the stepper motor.
#[derive(Clone, Copy, Debug)]
pub struct StepSegment {
    /// Timer cycles to wait before the next step.
    pub interval_ticks: u32,
    /// State of the direction pin (true = forward, false = backward).
    pub direction: bool,
    /// Active enable lines bitmask.
    pub enable_mask: u8,
}

/// A lock-free, single-producer single-consumer stepper controller.
pub struct StepperController<const N: usize> {
    /// The lock-free queue containing the step segments.
    queue: Queue<StepSegment, N>,
    /// The currently active direction state to avoid redundant writes.
    current_dir: bool,
    /// The bitmask representing the step pin to toggle.
    step_pin_mask: u32,
    /// The bitmask representing the direction pin.
    dir_pin_mask: u32,
}

impl<const N: usize> StepperController<N> {
    /// Creates a new StepperController with the specified pin masks.
    pub const fn new(step_pin_mask: u32, dir_pin_mask: u32) -> Self {
        Self {
            queue: Queue::new(),
            current_dir: false,
            step_pin_mask,
            dir_pin_mask,
        }
    }

    /// Pushes a new step segment into the lock-free queue without blocking.
    ///
    /// Returns an error if the queue is full.
    pub fn enqueue_segment(&mut self, segment: StepSegment) -> Result<(), &'static str> {
        self.queue.enqueue(segment).map_err(|_| "Stepper queue is full")
    }

    /// Executes the next step in the queue, writing directly to hardware registers.
    ///
    /// # Safety
    /// This function performs raw pointer writes to hardware registers. The caller
    /// must ensure that `bsrr_ptr` (Bit Set/Reset Register) and `arr_ptr` (Auto-Reload
    /// Register) point to valid memory locations for the target MCU peripherals.
    #[inline(always)]
    pub unsafe fn execute_next_step_isr(&mut self, bsrr_ptr: *mut u32, arr_ptr: *mut u32) {
        if let Some(segment) = self.queue.dequeue() {
            // Atomic evaluation and application of direction changes.
            if self.current_dir != segment.direction {
                self.current_dir = segment.direction;

                // BSRR (Bit Set/Reset Register):
                // Lower 16 bits set the pin, upper 16 bits reset the pin.
                let val = if segment.direction {
                    self.dir_pin_mask
                } else {
                    self.dir_pin_mask << 16
                };

                // Write the direction bit-set state to the raw BSRR address.
                core::ptr::write_volatile(bsrr_ptr, val);
            }

            // Write interval_ticks directly to the Timer Auto-Reload (ARR) address.
            core::ptr::write_volatile(arr_ptr, segment.interval_ticks);

            // Toggle the step pin with minimum clock-cycle latency via BSRR.
            // First, set the step pin high.
            core::ptr::write_volatile(bsrr_ptr, self.step_pin_mask);

            // Note: In a real implementation, a small delay or a separate timer match
            // might be needed here depending on the motor driver's required pulse width.
            // For this implementation, we immediately reset it for minimum latency.

            // Then, set the step pin low (reset).
            core::ptr::write_volatile(bsrr_ptr, self.step_pin_mask << 16);
        }
    }
}
