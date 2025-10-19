// File: crates/motion/src/profile.rs
//! Motion profiles for advanced features.

/// Pressure Advance model.
///
/// This model adjusts extruder step timing to compensate for pressure
/// buildup in the hotend, reducing ooze and improving corner quality.
#[derive(Debug, Copy, Clone, PartialEq)]
pub struct PressureAdvance {
    /// The pressure advance coefficient (in seconds).
    pub coefficient: f64,
}

impl PressureAdvance {
    /// Calculate the extra time needed for an extruder step.
    pub fn get_advance_time(&self, extruder_velocity: f64) -> f64 {
        self.coefficient * extruder_velocity
    }
}


/// Input Shaping model.
///
/// Input shaping is used to cancel out ringing/ghosting artifacts by
/// creating a specially shaped velocity profile that avoids exciting
/// the machine's resonant frequencies.
#[derive(Debug, Default, Copy, Clone)]
pub struct InputShaper {
    pub shaper_type: ShaperType,
    pub frequency: f32,
    // Cached impulses for the shaper
    impulses: [(f32, f32); 5], // (time_offset, amplitude)
    num_impulses: usize,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum ShaperType {
    #[default]
    None,
    ZV,
    MZV, // Minimum Zero Vibration
    EI,  // Extra-Insensitive
}

impl InputShaper {
    pub fn new(shaper_type: ShaperType, frequency: f32, damping_ratio: f32) -> Self {
        let mut shaper = Self {
            shaper_type,
            frequency,
            impulses: [(0.0, 0.0); 5],
            num_impulses: 0,
        };
        shaper.recalculate_impulses(damping_ratio);
        shaper
    }

    /// Recalculates the impulse train for the selected shaper.
    pub fn recalculate_impulses(&mut self, damping_ratio: f32) {
        // ... [Complex math to calculate impulse timings and amplitudes] ...
        // This is a placeholder for the actual shaper calculation logic.
        match self.shaper_type {
            ShaperType::ZV => {
                self.impulses[0] = (0.0, 1.0);
                self.num_impulses = 1;
            }
            _ => self.num_impulses = 0,
        }
    }

    /// Applies the input shaper to a given step interval.
    /// This is a simplified conceptual implementation. A real one would
    //  operate on a sequence of steps.
    pub fn apply(&self, original_delta_t: f64) -> f64 {
        // A real implementation would convolve the original step train
        // with the stored `impulses`. This is a placeholder.
        original_delta_t
    }
}

