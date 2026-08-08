
pub enum LaserMode {
    ContinuousVector,
    Dot,
}

pub struct LaserModeHandler {
    pub mode: LaserMode,
}

impl LaserModeHandler {
    pub fn set_mode(&mut self, mode: LaserMode) {
        self.mode = mode;
    }

    pub async fn execute_dot(&mut self, x: f32, y: f32, power: f32, duration_us: u32) {
        // In a real implementation, this would move to X/Y, stop, fire a pulse, and then be ready for the next move.
        // log::info!("Firing dot at ({}, {}) with power {} for {}us", x, y, power, duration_us);
    }
}
