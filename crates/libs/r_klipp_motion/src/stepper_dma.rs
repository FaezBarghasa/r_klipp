use core::sync::atomic::{AtomicU8, Ordering};

pub struct DmaStepEngine {
    buffers: [[u32; 256]; 2],
    active_buffer: AtomicU8,
    trigger_points: heapless::Vec<(u32, u8), 16>, // (step_count, gpio_pin)
}

impl DmaStepEngine {
    pub fn new() -> Self {
        Self {
            buffers: [[0; 256]; 2],
            active_buffer: AtomicU8::new(0),
            trigger_points: heapless::Vec::new(),
        }
    }

    pub fn add_trigger_point(&mut self, step_count: u32, gpio_pin: u8) {
        self.trigger_points.push((step_count, gpio_pin)).unwrap();
    }

    // This would be called from the DMA ISR
    pub fn check_triggers(&self, current_step: u32) {
        for (step_count, gpio_pin) in &self.trigger_points {
            if *step_count == current_step {
                // Trigger GPIO pin via hardware timer one-shot
            }
        }
    }

    pub fn swap_buffer(&mut self) {
        let current = self.active_buffer.load(Ordering::Relaxed);
        let next = 1 - current;
        self.active_buffer.store(next, Ordering::Release);
    }

    pub fn get_inactive_buffer_mut(&mut self) -> &mut [u32; 256] {
        let inactive_index = 1 - self.active_buffer.load(Ordering::Acquire);
        &mut self.buffers[inactive_index as usize]
    }
}
