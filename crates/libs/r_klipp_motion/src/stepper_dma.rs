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
        let current_buffer = self.active_buffer.load(Ordering::Acquire);
        let next_buffer = 1 - current_buffer;

        // The main loop would fill the inactive buffer `self.buffers[next_buffer as usize]` here.

        self.active_buffer.store(next_buffer, Ordering::Release);
    }

    // This would be called from a DMA interrupt handler
    pub fn get_active_buffer(&self) -> &[u32] {
        let active_buffer_index = self.active_buffer.load(Ordering::Acquire);
        &self.buffers[active_buffer_index as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::thread;

    #[test]
    fn test_atomic_buffer_swap_consistency() {
        let engine = Arc::new(std::sync::Mutex::new(DmaStepEngine::new()));
        let engine_clone = engine.clone();

        let main_loop = thread::spawn(move || {
            for _ in 0..1000 {
                let mut engine = engine.lock().unwrap();
                engine.swap_buffer();
            }
        });

        let isr_simulation = thread::spawn(move || {
            for _ in 0..1000 {
                let engine = engine_clone.lock().unwrap();
                let _ = engine.get_active_buffer();
            }
        });

        main_loop.join().unwrap();
        isr_simulation.join().unwrap();

        // The test passes if it completes without panicking, indicating no data races.
    }
}