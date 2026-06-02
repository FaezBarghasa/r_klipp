// crates/mcu-drivers/src/stepper.rs
#![no_std]

use heapless::spsc::Queue;

/// Representation of a single discrete stepping chunk dispatched to step timers.
#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct StepSegment {
    /// Number of local master clock cycles to wait before toggling the step pin.
    pub interval_ticks: u32,
    /// Digital state of the direction pin (true = high, false = low).
    pub direction: bool,
    /// Bitmask of stepper motor enable lines to set/clear.
    pub enable_mask: u8,
}

pub struct StepperController<const N: usize> {
    /// Lock-free ring buffer holding incoming calculated steps.
    pub queue: Queue<StepSegment, N>,
    /// Track current hardware direction pin state to prevent unnecessary GPIO writes.
    pub current_dir: bool,
    /// Target step pin bitmask for fast GPIO port writes.
    pub step_pin_mask: u32,
}

impl<const N: usize> StepperController<N> {
    pub const fn new(step_pin_mask: u32) -> Self {
        Self {
            queue: Queue::new(),
            current_dir: false,
            step_pin_mask,
        }
    }

    /// Executed by the communications task (Priority 1) to push calculated steps into the queue.
    #[inline(always)]
    pub fn enqueue_segment(&mut self, segment: StepSegment) -> Result<(), &'static str> {
        self.queue.enqueue(segment).map_err(|_| "Error: Step queue is full. Real-time desync imminent.")
    }

    /// High-Priority ISR Callback (Priority 5).
    /// Updates physical output registers and configures the timer reload register for the next step.
    /// 
    /// # Arguments
    /// * `bsrr_register` - Raw pointer to GPIO bit set/reset register (e.g., STM32 BSRR).
    /// * `arr_register` - Raw pointer to the auto-reload match register of the timer.
    #[inline(always)]
    pub unsafe fn execute_next_step_isr(
        &mut self, 
        bsrr_register: *mut u32, 
        arr_register: *mut u32
    ) {
        if let Some(segment) = self.queue.dequeue() {
            // Write direction pin state directly to hardware register
            if segment.direction != self.current_dir {
                self.current_dir = segment.direction;
                if self.current_dir {
                    // Set direction pin high
                    *bsrr_register = self.step_pin_mask;
                } else {
                    // Reset direction pin low
                    *bsrr_register = self.step_pin_mask << 16;
                }
            }
            
            // Set step pin high
            *bsrr_register = self.step_pin_mask;
            
            // Write next step interval time directly into hardware auto-reload register
            *arr_register = segment.interval_ticks;
            
            // Clear step pin (creates a pulse of minimum width based on CPU clock cycle speed)
            *bsrr_register = self.step_pin_mask << 16;
        } else {
            // Underflow fallback: No moves in queue, safely halt timer
            *arr_register = u32::MAX;
        }
    }
}
