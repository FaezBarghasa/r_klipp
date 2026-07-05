use core::sync::atomic::{AtomicU8, Ordering};
use embassy_sync::blocking_mutex::raw::ThreadModeRawMutex;
use embassy_sync::channel::Channel;
use r_klipp_api::hal::DmaStepEngine;

pub struct StepperDma {
    buffers: [[u32; 256]; 2],
    active_buffer: AtomicU8,
    dma_channel: Channel<ThreadModeRawMutex, &'static [u32], 1>,
}

impl StepperDma {
    pub fn new() -> Self {
        Self {
            buffers: [[0; 256]; 2],
            active_buffer: AtomicU8::new(0),
            dma_channel: Channel::new(),
        }
    }

    pub fn get_dma_channel(&self) -> &Channel<ThreadModeRawMutex, &'static [u32], 1> {
        &self.dma_channel
    }
}

impl DmaStepEngine for StepperDma {
    async fn stream_steps(&mut self, buffer: &[u32]) {
        let inactive_buffer_idx = self.active_buffer.load(Ordering::Acquire) ^ 1;
        self.buffers[inactive_buffer_idx as usize][..buffer.len()].copy_from_slice(buffer);
    }

    fn swap_buffer(&mut self) {
        let current_buffer = self.active_buffer.load(Ordering::Acquire);
        self.dma_channel.try_send(&self.buffers[current_buffer as usize]).unwrap();
        self.active_buffer.fetch_xor(1, Ordering::Release);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::sync::atomic::Ordering;
    use std::thread;
    use std::time::Duration;

    #[test]
    fn test_atomic_buffer_swap_race_condition() {
        let mut engine = StepperDma::new();
        let engine_ptr = &mut engine as *mut _;

        let handle = thread::spawn(move || {
            let engine = unsafe { &mut *engine_ptr };
            for i in 0..100 {
                let buffer = [i as u32; 256];
                // In a real async context, this would be an await point
                // Here we simulate the main loop filling the buffer
                let inactive_buffer_idx = engine.active_buffer.load(Ordering::Acquire) ^ 1;
                engine.buffers[inactive_buffer_idx as usize].copy_from_slice(&buffer);
                engine.active_buffer.fetch_xor(1, Ordering::Release);
                thread::sleep(Duration::from_micros(10));
            }
        });

        for i in 0..100 {
            // Simulate the DMA ISR consuming the buffer
            let current_buffer_idx = engine.active_buffer.load(Ordering::Acquire);
            let buffer_content = engine.buffers[current_buffer_idx as usize][0];

            // The buffer content should be consistent, not a mix of i and i-1
            let first_val = engine.buffers[current_buffer_idx as usize][0];
            for val in &engine.buffers[current_buffer_idx as usize] {
                assert_eq!(*val, first_val);
            }
            thread::sleep(Duration::from_micros(10));
        }

        handle.join().unwrap();
    }
}
