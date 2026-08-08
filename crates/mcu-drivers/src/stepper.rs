use heapless::spsc::Queue;

/// Segment definition for the stepper motor queue.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepSegment {
    pub interval_ticks: u32,
    pub direction: bool,
    pub enable_mask: u8,
}

/// Lock-free Single-Producer Single-Consumer (SPSC) Stepper Motor Engine.
pub struct StepperController<const N: usize> {
    queue: Queue<StepSegment, N>,
    current_dir: bool,
    step_pin_mask: u32,
    dir_pin_mask: u32,
}

impl<const N: usize> StepperController<N> {
    /// Creates a new, statically allocated stepper controller.
    pub const fn new(step_pin_mask: u32, dir_pin_mask: u32) -> Self {
        Self {
            queue: Queue::new(),
            current_dir: false,
            step_pin_mask,
            dir_pin_mask,
        }
    }

    /// Pushes a new step segment into the queue without blocking.
    pub fn enqueue_segment(&mut self, segment: StepSegment) -> Result<(), &'static str> {
        self.queue.enqueue(segment).map_err(|_| "Queue is full")
    }

    /// Dequeues a segment from the queue.
    pub fn dequeue_segment(&mut self) -> Option<StepSegment> {
        self.queue.dequeue()
    }

    /// Executes the next step from the queue. Designed to be called inside a high-priority ISR.
    /// 
    /// # Safety
    /// This function performs raw pointer writes to hardware memory.
    /// `bsrr_ptr` must point to a valid GPIO Port Bit Set/Reset Register.
    /// `arr_ptr` must point to a valid Timer Auto-Reload Register.
    pub unsafe fn execute_next_step_isr(&mut self, bsrr_ptr: *mut u32, arr_ptr: *mut u32) {
        if let Some(segment) = self.queue.dequeue() {
            // Atomic evaluation of direction changes
            if self.current_dir != segment.direction {
                self.current_dir = segment.direction;
                
                // Immediate writing of the direction bit-set state to the raw BSRR address
                // Hardware layout mapping: lower 16 bits set the pin, upper 16 bits reset the pin.
                if self.current_dir {
                    core::ptr::write_volatile(bsrr_ptr, self.dir_pin_mask);
                } else {
                    core::ptr::write_volatile(bsrr_ptr, self.dir_pin_mask << 16);
                }
            }

            // Writing interval_ticks directly to the Timer Auto-Reload ARR address
            core::ptr::write_volatile(arr_ptr, segment.interval_ticks);

            // Toggling the step pin with minimum clock-cycle latency via BSRR
            // Set the step pin high
            core::ptr::write_volatile(bsrr_ptr, self.step_pin_mask);
            
            // Reset the step pin low immediately to generate the minimum step pulse
            core::ptr::write_volatile(bsrr_ptr, self.step_pin_mask << 16);
        }
    }
}