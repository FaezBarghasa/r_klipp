// crates/mcu-drivers/src/stepper_dma.rs

/// Direct Memory Access (DMA) Double Buffer configuration.
pub struct DmaStepEngine {
    /// Memory blocks mapping step pulse interval timings
    buffer_a: [u32; 256],
    buffer_b: [u32; 256],
    active_buffer: u8,
}

impl DmaStepEngine {
    pub const fn new() -> Self {
        Self {
            buffer_a: [1000; 256], // Standard baseline idle steps (ticks)
            buffer_b: [1000; 256],
            active_buffer: 0,
        }
    }

    /// Populates idle buffers with calculated move step counts.
    pub fn update_inactive_buffer(&mut self, source_steps: &[u32]) {
        let buffer = if self.active_buffer == 0 {
            &mut self.buffer_b
        } else {
            &mut self.buffer_a
        };
        
        let len = source_steps.len().min(256);
        buffer[..len].copy_from_slice(&source_steps[..len]);
        
        // Zero-fill remaining buffer spaces to safe default values
        if len < 256 {
            buffer[len..].fill(u32::MAX); // Maximum timer interval (idle/paused)
        }
    }

    /// Executed within the half-transfer (HT) or complete-transfer (TC) DMA interrupts.
    /// Updates index and alternates memory access.
    pub fn handle_dma_interrupt(&mut self) -> *const u32 {
        self.active_buffer ^= 1;
        if self.active_buffer == 0 {
            self.buffer_a.as_ptr()
        } else {
            self.buffer_b.as_ptr()
        }
    }
}
