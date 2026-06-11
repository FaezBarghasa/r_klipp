//! Input shaper implementation – Zero‑Vibration (ZV) FIR filter.
//!
//! The shaper splits a raw step pulse into two delayed pulses according to the
//! classic ZV formulas:
//!
//! ```text
//! A₁ = 1 / (1 + K)
//! A₂ = K / (1 + K)
//! t₁ = 0
//! t₂ = 1 / (2·f_d)
//! K   = exp(-ζ·π / sqrt(1‑ζ²))
//! ```
//!
//! where `ζ` is the damping ratio and `f_d` is the dominant resonance frequency
//! (Hz). The implementation is fully static – no heap allocation – by using
//! `smallvec` for the return type.

use smallvec::SmallVec;

/// Configuration for a ZV input shaper.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ZVConfig {
    /// Damping ratio (typically 0.0 – 0.2 for 3‑D printers).
    pub damping_ratio: f64,
    /// Dominant resonance frequency in Hertz.
    pub resonance_hz: f64,
}

impl ZVConfig {
    /// Compute the coefficient `K` used in the amplitude split.
    #[inline]
    pub fn k_coeff(&self) -> f64 {
        // Guard against invalid parameters.
        if self.damping_ratio <= 0.0 || self.resonance_hz <= 0.0 {
            return 0.0;
        }
        (-self.damping_ratio * std::f64::consts::PI
            / (1.0 - self.damping_ratio * self.damping_ratio).sqrt())
            .exp()
    }

    /// Compute the second‑pulse delay in microseconds.
    #[inline]
    pub fn second_pulse_delay_us(&self) -> u64 {
        // f_d is in Hz, so period = 1/f_d seconds. Half‑period => 1/(2·f_d).
        // Convert seconds to microseconds.
        let period_sec = 1.0 / (2.0 * self.resonance_hz);
        (period_sec * 1_000_000.0).round() as u64
    }
}

/// A zero‑vibration input shaper that operates on raw step timestamps.
#[derive(Debug, Clone, PartialEq)]
pub struct InputShaper {
    config: ZVConfig,
    /// Pre‑computed amplitudes for speed.
    a1: f64,
    a2: f64,
    /// Pre‑computed delay for the second pulse.
    delay_us: u64,
}

impl InputShaper {
    /// Create a new shaper from a configuration.
    pub fn new(config: ZVConfig) -> Self {
        let k = config.k_coeff();
        let a1 = 1.0 / (1.0 + k);
        let a2 = k / (1.0 + k);
        let delay_us = config.second_pulse_delay_us();
        InputShaper { config, a1, a2, delay_us }
    }

    /// Apply the shaper to a raw step interval.
    ///
    /// * `raw_step_us` – the original step interval expressed in microseconds.
    ///
    /// Returns a `SmallVec` containing up to two delayed step intervals. The
    /// first entry is always `raw_step_us * a1`. The second entry (if `a2 > 0`) is
    /// delayed by `delay_us` and scaled by `a2`.
    pub fn shape_step_timing(&self, raw_step_us: u64) -> SmallVec<[u64; 4]> {
        // Convert the raw interval to f64 for precise multiplication.
        let raw_f = raw_step_us as f64;
        let mut out = SmallVec::<[u64; 4]>::new();
        // First pulse – immediate.
        let first = (raw_f * self.a1).round() as u64;
        out.push(first);
        // Second pulse – only if the amplitude is non‑zero.
        if self.a2 > 0.0 {
            let second = (raw_f * self.a2).round() as u64;
            // Encode delay by adding the delay to the timestamp when the caller
            // schedules the pulse. Here we simply store the interval; the caller
            // must apply the `delay_us` when emitting the second pulse.
            // The API returns the raw interval; the consumer can later add the
            // delay as needed.
            out.push(second);
        }
        out
    }

    /// Return the pre‑computed delay between the two pulses (microseconds).
    /// The caller can add this delay to the timestamp of the second pulse.
    #[inline]
    pub fn delay_us(&self) -> u64 {
        self.delay_us
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shaper_basic() {
        let cfg = ZVConfig {
            damping_ratio: 0.05,
            resonance_hz: 80.0,
        };
        let shaper = InputShaper::new(cfg);
        let raw = 1000u64; // 1 ms step interval
        let pulses = shaper.shape_step_timing(raw);
        assert_eq!(pulses.len(), 2);
        // Amplitudes must sum to roughly 1.
        let sum: f64 = pulses[0] as f64 + pulses[1] as f64;
        assert!((sum - raw as f64).abs() < 1.0);
        // Verify delay is reasonable.
        assert!(shaper.delay_us() > 0);
    }
}
