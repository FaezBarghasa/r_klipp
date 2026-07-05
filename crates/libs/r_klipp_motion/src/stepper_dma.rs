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
    use std::time::Duration;

    #[test]
    fn test_atomic_buffer_swap_consistency() {
        let engine = Arc::new(DmaStepEngine::new());
        let engine_clone_main = engine.clone();
        let engine_clone_isr = engine.clone();

        let main_loop = thread::spawn(move || {
            let mut engine_mut = Arc::into_inner(engine_clone_main).unwrap();
            for i in 0..1000 {
                engine_mut.swap_buffer();
                // Simulate work
                thread::sleep(Duration::from_micros(10));
            }
        });

        let isr_simulation = thread::spawn(move || {
            for _ in 0..1000 {
                let _ = engine_clone_isr.get_active_buffer();
                // Simulate ISR execution
                thread::sleep(Duration::from_micros(5));
            }
        });

        main_loop.join().unwrap();
        isr_simulation.join().unwrap();

        // The test passes if it completes without panicking, indicating no data races.
    }
}