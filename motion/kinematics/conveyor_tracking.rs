//! Real-time conveyor tracking.

use core::sync::atomic::{AtomicI32, Ordering};

pub struct ConveyorTracker {
    conveyor_pos_atomic: AtomicI32,
    encoder_resolution: f32, // counts per mm
}

impl ConveyorTracker {
    pub fn new(encoder_resolution: f32) -> Self {
        Self {
            conveyor_pos_atomic: AtomicI32::new(0),
            encoder_resolution,
        }
    }

    pub fn update_encoder_count(&self, count: i32) {
        self.conveyor_pos_atomic.store(count, Ordering::Relaxed);
    }

    pub fn get_target_pos(&self, programmed_pos: &[f32; 3]) -> [f32; 3] {
        let conveyor_pos = self.conveyor_pos_atomic.load(Ordering::Relaxed) as f32 / self.encoder_resolution;
        [
            programmed_pos[0],
            programmed_pos[1] + conveyor_pos,
            programmed_pos[2],
        ]
    }
}
