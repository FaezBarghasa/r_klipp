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

    /// Returns the raw pointer to the currently active buffer.
    pub fn get_active_buffer_ptr(&self) -> *const u32 {
        if self.active_buffer == 0 {
            self.buffer_a.as_ptr()
        } else {
            self.buffer_b.as_ptr()
        }
    }

    /// Checks if Buffer A is the active buffer.
    pub fn is_buffer_a_active(&self) -> bool {
        self.active_buffer == 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dma_stepper_double_buffering() {
        let mut engine = DmaStepEngine::new();
        
        // Initial state: buffer A is active (active_buffer = 0)
        assert!(engine.is_buffer_a_active());
        assert_eq!(engine.get_active_buffer_ptr(), engine.buffer_a.as_ptr());

        // Update inactive buffer (which should be buffer B)
        let source = [500u32; 10];
        engine.update_inactive_buffer(&source);
        
        // Verify buffer B was updated
        assert_eq!(engine.buffer_b[0], 500);
        assert_eq!(engine.buffer_b[9], 500);
        assert_eq!(engine.buffer_b[10], u32::MAX); // Zero-filled to max
        
        // Trigger DMA interrupt, swapping active buffers to buffer B (active_buffer = 1)
        let next_ptr = engine.handle_dma_interrupt();
        assert_eq!(next_ptr, engine.buffer_b.as_ptr());
        assert!(!engine.is_buffer_a_active());

        // Update inactive buffer (which should be buffer A now)
        let source_a = [300u32; 5];
        engine.update_inactive_buffer(&source_a);

        // Verify buffer A was updated
        assert_eq!(engine.buffer_a[0], 300);
        assert_eq!(engine.buffer_a[4], 300);
        assert_eq!(engine.buffer_a[5], u32::MAX);
    }
}
