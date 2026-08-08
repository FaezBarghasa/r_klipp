use core::sync::atomic::{AtomicU8, Ordering};

pub struct DmaStepEngine {
    pub buffer_a: [u32; 256],
    pub buffer_b: [u32; 256],
    pub active_buffer: AtomicU8,
}

impl DmaStepEngine {
    pub fn new() -> Self {
        Self {
            buffer_a: [u32::MAX; 256],
            buffer_b: [u32::MAX; 256],
            active_buffer: AtomicU8::new(0),
        }
    }

    pub fn update_inactive_buffer(&mut self, source_steps: &[u32]) {
        let current_active_buffer = self.active_buffer.load(Ordering::Acquire);
        let target_buffer = match current_active_buffer {
            0 => &mut self.buffer_b,
            _ => &mut self.buffer_a,
        };

        let copy_len = if source_steps.len() > 256 { 256 } else { source_steps.len() };
        
        target_buffer[..copy_len].copy_from_slice(&source_steps[..copy_len]);

        for item in target_buffer.iter_mut().skip(copy_len) {
            *item = u32::MAX;
        }
    }

    pub fn handle_dma_interrupt(&mut self) -> *const u32 {
        let current_active_buffer = self.active_buffer.load(Ordering::Acquire);
        let next_active_buffer = if current_active_buffer == 0 { 1 } else { 0 };
        self.active_buffer.store(next_active_buffer, Ordering::Release);

        match next_active_buffer {
            0 => self.buffer_a.as_ptr(),
            _ => self.buffer_b.as_ptr(),
        }
    }
}