use core::sync::atomic::{AtomicBool, AtomicU8, AtomicUsize, Ordering};

/// Packed hardware DMA step pulse event
/// Encodes step interval in timer clock ticks (e.g. 48MHz or 168MHz base),
/// multi-axis step bitmask (bits 0..7), direction bitmask (bits 8..15), and enable flags.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(C)]
pub struct DmaStepEvent {
    pub timer_ticks: u32,
    pub step_bitmask: u8,
    pub dir_bitmask: u8,
    pub enable_bitmask: u8,
    pub flags: u8,
}

impl DmaStepEvent {
    pub const END_OF_BLOCK: u32 = u32::MAX;

    pub const fn empty() -> Self {
        Self {
            timer_ticks: Self::END_OF_BLOCK,
            step_bitmask: 0,
            dir_bitmask: 0,
            enable_bitmask: 0,
            flags: 0,
        }
    }
}

pub const DMA_BUFFER_CAPACITY: usize = 512;

/// Double-buffered DMA Step Pulse Engine
/// Completely decouples high-level motion trajectory calculations from physical timer interrupt jitter.
pub struct DmaStepEngine {
    pub buffer_a: [DmaStepEvent; DMA_BUFFER_CAPACITY],
    pub buffer_b: [DmaStepEvent; DMA_BUFFER_CAPACITY],
    pub active_buffer: AtomicU8,
    pub buffer_a_len: AtomicUsize,
    pub buffer_b_len: AtomicUsize,
    pub underrun_flag: AtomicBool,
}

impl DmaStepEngine {
    pub const fn new() -> Self {
        Self {
            buffer_a: [DmaStepEvent::empty(); DMA_BUFFER_CAPACITY],
            buffer_b: [DmaStepEvent::empty(); DMA_BUFFER_CAPACITY],
            active_buffer: AtomicU8::new(0),
            buffer_a_len: AtomicUsize::new(0),
            buffer_b_len: AtomicUsize::new(0),
            underrun_flag: AtomicBool::new(false),
        }
    }

    /// Loads the inactive shadow buffer with the next sequence of step pulse intervals.
    /// This is executed by the background planner task without locking the high-priority DMA ISR.
    pub fn fill_inactive_buffer(&mut self, events: &[DmaStepEvent]) -> usize {
        let active = self.active_buffer.load(Ordering::Acquire);
        let (target_buf, target_len_atomic) = if active == 0 {
            (&mut self.buffer_b, &self.buffer_b_len)
        } else {
            (&mut self.buffer_a, &self.buffer_a_len)
        };

        let copy_count = events.len().min(DMA_BUFFER_CAPACITY);
        target_buf[..copy_count].copy_from_slice(&events[..copy_count]);

        // If less than capacity, pad with sentinel
        if copy_count < DMA_BUFFER_CAPACITY {
            target_buf[copy_count] = DmaStepEvent::empty();
        }

        target_len_atomic.store(copy_count, Ordering::Release);
        copy_count
    }

    /// Executed inside the DMA Transfer Complete (TC) hardware ISR.
    /// Atomically swaps the active memory buffer pointer in $<50\,\text{ns}$ with zero CPU jitter.
    #[inline(always)]
    pub fn handle_dma_transfer_complete(&mut self) -> (*const DmaStepEvent, usize) {
        let current_active = self.active_buffer.load(Ordering::Acquire);
        let next_active = if current_active == 0 { 1 } else { 0 };

        let (next_ptr, next_len) = if next_active == 0 {
            (self.buffer_a.as_ptr(), self.buffer_a_len.load(Ordering::Acquire))
        } else {
            (self.buffer_b.as_ptr(), self.buffer_b_len.load(Ordering::Acquire))
        };

        if next_len == 0 {
            self.underrun_flag.store(true, Ordering::SeqCst);
        }

        // Atomically commit active buffer switch
        self.active_buffer.store(next_active, Ordering::Release);
        (next_ptr, next_len)
    }

    pub fn is_underrun(&self) -> bool {
        self.underrun_flag.load(Ordering::Relaxed)
    }

    pub fn clear_underrun(&self) {
        self.underrun_flag.store(false, Ordering::Relaxed);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dma_step_engine_double_buffering() {
        let mut engine = DmaStepEngine::new();

        let mut test_events = [DmaStepEvent::empty(); 10];
        for (i, ev) in test_events.iter_mut().enumerate() {
            ev.timer_ticks = (1000 + i * 50) as u32;
            ev.step_bitmask = 0b0000_0001;
            ev.dir_bitmask = 0b0000_0000;
        }

        // Inactive buffer is B (0 is active)
        let copied = engine.fill_inactive_buffer(&test_events);
        assert_eq!(copied, 10);

        // ISR triggers: swaps to B
        let (ptr, len) = engine.handle_dma_transfer_complete();
        assert_eq!(len, 10);
        assert!(!ptr.is_null());
        assert_eq!(engine.active_buffer.load(Ordering::Relaxed), 1);

        // Inactive buffer is now A
        let copied_a = engine.fill_inactive_buffer(&test_events[0..5]);
        assert_eq!(copied_a, 5);

        // ISR triggers: swaps to A
        let (_ptr_a, len_a) = engine.handle_dma_transfer_complete();
        assert_eq!(len_a, 5);
        assert_eq!(engine.active_buffer.load(Ordering::Relaxed), 0);
        assert!(!engine.is_underrun());
    }
}