// File: crates/motion/src/profile.rs
//! Motion profiles for advanced features.
#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
use libm::{acosf, cosf, expf, sqrtf};
#[cfg(feature = "std")]
use std::primitive::f32::{acos as acosf, cos, exp as expf, sqrt as sqrtf};

/// Pressure Advance model.
///
/// This model adjusts extruder step timing to compensate for pressure
/// buildup in the hotend, reducing ooze and improving corner quality.
#[derive(Debug, Copy, Clone, PartialEq, Default)]
pub struct PressureAdvance {
    /// The pressure advance coefficient (in seconds).
    pub coefficient: f32,
    /// An optional smoothing time for the pressure advance adjustment.
    pub smooth_time: f32,
}

impl PressureAdvance {
    /// Creates a new PressureAdvance configuration.
    pub fn new(coefficient: f32, smooth_time: f32) -> Self {
        Self {
            coefficient,
            smooth_time,
        }
    }

    /// Calculate the extra steps needed for an extruder based on velocity and acceleration.
    /// This is based on Klipper's "step on pressure" model.
    pub fn get_advance_steps(&self, velocity: f32, accel: f32, steps_per_mm: f32) -> f32 {
        let pa_time = self.coefficient + self.smooth_time * velocity * 0.5;
        (pa_time * velocity + pa_time * pa_time * accel * 0.5) * steps_per_mm
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
    // Cached impulses for the shaper: (time_offset, amplitude)
    pub impulses: [(f32, f32); 5],
    pub num_impulses: usize,
    /// The duration of the shaper, from the first to the last impulse.
    pub duration: f32,
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
pub enum ShaperType {
    #[default]
    None,
    ZV,  // Zero Vibration
    MZV, // Minimum Zero Vibration
    EI,  // Extra-Insensitive
    ZVD, // Zero Vibration and Derivative
}

impl InputShaper {
    /// Creates a new Input Shaper and calculates its impulse train.
    pub fn new(shaper_type: ShaperType, frequency: f32, damping_ratio: f32) -> Self {
        let mut shaper = Self {
            shaper_type,
            frequency,
            impulses: [(0.0, 0.0); 5],
            num_impulses: 0,
            duration: 0.0,
        };
        if frequency > 0.0 {
            shaper.recalculate_impulses(damping_ratio);
        }
        shaper
    }

    /// Recalculates the impulse train for the selected shaper.
    /// The formulas are derived from standard vibration control theory.
    pub fn recalculate_impulses(&mut self, damping_ratio: f32) {
        if self.frequency <= 0.0 {
            self.num_impulses = 0;
            self.duration = 0.0;
            return;
        }

        let pi = core::f32::consts::PI;
        let shaper_period = 1.0 / self.frequency;
        let damping_ratio = damping_ratio.max(0.0);

        let k = if damping_ratio < 1.0 {
            expf(-damping_ratio * pi / sqrtf(1.0 - damping_ratio * damping_ratio))
        } else {
            0.0
        };

        match self.shaper_type {
            ShaperType::None => {
                self.num_impulses = 0;
            }
            ShaperType::ZV => {
                let a1 = 1.0 / (1.0 + k);
                let a2 = k * a1;
                self.impulses[0] = (0.0, a1);
                self.impulses[1] = (0.5 * shaper_period, a2);
                self.num_impulses = 2;
            }
            ShaperType::MZV => {
                let a1 = 1.0 / (1.0 + 2.0 * k + k * k);
                let a2 = 2.0 * k * a1;
                let a3 = k * k * a1;
                self.impulses[0] = (0.0, a1);
                self.impulses[1] = (0.5 * shaper_period, a2);
                self.impulses[2] = (shaper_period, a3);
                self.num_impulses = 3;
            }
            ShaperType::EI => {
                let v = if damping_ratio < 1.0 {
                    let phi = acosf(damping_ratio);
                    2.0 * pi * damping_ratio / (phi * sqrtf(1.0 - damping_ratio * damping_ratio))
                } else {
                    0.0
                };
                let w = 2.0 * pi * damping_ratio / (acosf(damping_ratio) * sqrtf(1.0 - damping_ratio*damping_ratio));
                let k2 = k*k;
                let a1 = 1.0 / (1.0 + 3.0 * k + 3.0 * k2 + k2 * k);
                let a2 = 3.0 * k * a1;
                let a3 = 3.0 * k2 * a1;
                let a4 = k2 * k * a1;
                self.impulses[0] = (0.0, a1);
                self.impulses[1] = (0.5 * shaper_period, a2);
                self.impulses[2] = (shaper_period, a3);
                self.impulses[3] = (1.5 * shaper_period, a4);
                self.num_impulses = 4;
            }
            ShaperType::ZVD => {
                let k2 = k*k;
                let a1 = 1.0 / (1.0 + 2.0*k + k2);
                let a2 = 2.0*k*a1;
                let a3 = k2*a1;
                self.impulses[0] = (0.0, a1);
                self.impulses[1] = (0.5*shaper_period, a2);
                self.impulses[2] = (shaper_period, a3);
                self.num_impulses = 3;
            }
        }
        if self.num_impulses > 0 {
            self.duration = self.impulses[self.num_impulses - 1].0;
        } else {
            self.duration = 0.0;
        }
    }
}
