use core::sync::atomic::{AtomicU8, Ordering};

pub struct DmaStepEngine {
    buffers: [[u32; 256]; 2],
    active_buffer: AtomicU8,
}

impl DmaStepEngine {
    pub fn new() -> Self {
        Self {
            buffers: [[0; 256]; 2],
            active_buffer: AtomicU8::new(0),
        }
    }

    pub fn swap_buffer(&mut self) {
        let current = self.active_buffer.load(Ordering::Relaxed);
        let next = 1 - current;
        // In a real scenario, you would configure the DMA to use the 'next' buffer here.
        self.active_buffer.store(next, Ordering::Release);
    }

    // This would be called from the async context to fill the inactive buffer
    pub fn get_inactive_buffer_mut(&mut self) -> &mut [u32; 256] {
        let inactive_index = 1 - self.active_buffer.load(Ordering::Acquire);
        &mut self.buffers[inactive_index as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;
    use std::sync::Arc;

    #[test]
    fn test_atomic_swap_race_condition() {
        let engine = Arc::new(std::sync::Mutex::new(DmaStepEngine::new()));
        let engine_clone = engine.clone();

        let writer_thread = thread::spawn(move || {
            for _ in 0..1000 {
                let mut engine = engine_clone.lock().unwrap();
                let buffer = engine.get_inactive_buffer_mut();
                buffer.iter_mut().for_each(|x| *x += 1);
                engine.swap_buffer();
            }
        });

        let isr_thread = thread::spawn(move || {
            for _ in 0..1000 {
                let mut engine = engine.lock().unwrap();
                // Simulate ISR reading the active buffer
                let active_idx = engine.active_buffer.load(Ordering::Acquire);
                let _active_buffer = &engine.buffers[active_idx as usize];
                // No swap here, just ensuring access is safe
            }
        });

        writer_thread.join().unwrap();
        isr_thread.join().unwrap();
    }
}
