#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Plane { G17, G18, G19 }

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Units { Millimeters, Inches }

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum Positioning { Absolute, Relative }

#[derive(Debug, PartialEq, Clone, Copy)]
pub struct ModalState {
    pub motion_mode: u16,
    pub plane: Plane,
    pub units: Units,
    pub positioning: Positioning,
    pub feed_rate: f32,
    pub spindle_speed: f32,
    pub tool: u16,
    // ... other modal states
}

impl Default for ModalState {
    fn default() -> Self {
        Self {
            motion_mode: 1, // G1
            plane: Plane::G17, // XY plane
            units: Units::Millimeters,
            positioning: Positioning::Absolute,
            feed_rate: 500.0,
            spindle_speed: 0.0,
            tool: 0,
        }
    }
}
