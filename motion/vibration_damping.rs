//! Active vibration damping using digital filters.
#![no_std]

use micromath::F32Ext;

/// A second-order IIR (Infinite Impulse Response) Notch Filter.
/// This is used to remove a specific frequency from a signal, which is
/// essential for suppressing machine resonance (chatter).
#[derive(Debug, Clone, Copy)]
pub struct IirNotchFilter {
    // Filter coefficients
    a: [f32; 3], // Feedback coefficients
    b: [f32; 3], // Feedforward coefficients
    // Filter state (history of inputs and outputs)
    x: [f32; 2], // Previous inputs [x(n-1), x(n-2)]
    y: [f32; 2], // Previous outputs [y(n-1), y(n-2)]
}

impl IirNotchFilter {
    /// Creates a new IIR Notch Filter for a given frequency.
    ///
    /// # Arguments
    /// * `sample_rate`: The rate at which the signal is sampled (e.g., 1000 Hz for a 1ms ISR).
    /// * `notch_freq`: The frequency to remove (e.g., the machine's resonant frequency).
    /// * `q_factor`: The quality factor of the filter. A higher Q gives a narrower notch.
    ///
    /// # Returns
    /// A new `IirNotchFilter` instance.
    pub fn new(sample_rate: f32, notch_freq: f32, q_factor: f32) -> Self {
        let w0 = 2.0 * core::f32::consts::PI * notch_freq / sample_rate;
        let alpha = w0.sin() / (2.0 * q_factor);

        let b0 = 1.0;
        let b1 = -2.0 * w0.cos();
        let b2 = 1.0;

        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha;

        // Normalize the coefficients so that the first 'a' coefficient is 1.
        Self {
            b: [b0 / a0, b1 / a0, b2 / a0],
            a: [1.0, a1 / a0, a2 / a0],
            x: [0.0, 0.0],
            y: [0.0, 0.0],
        }
    }

    /// Applies the filter to a single input sample.
    /// This implements the direct form II transposed structure for IIR filters,
    /// which is efficient and numerically stable.
    /// y(n) = b0*x(n) + b1*x(n-1) + b2*x(n-2) - a1*y(n-1) - a2*y(n-2)
    pub fn apply(&mut self, input: f32) -> f32 {
        let output = self.b[0] * input + self.b[1] * self.x[0] + self.b[2] * self.x[1]
                   - self.a[1] * self.y[0] - self.a[2] * self.y[1];

        // Update state history
        self.x[1] = self.x[0];
        self.x[0] = input;
        self.y[1] = self.y[0];
        self.y[0] = output;

        output
    }

    /// Resets the filter's internal state.
    pub fn reset(&mut self) {
        self.x = [0.0, 0.0];
        self.y = [0.0, 0.0];
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Generates a sine wave signal with a given frequency and sample rate.
    fn generate_sine_wave(freq: f32, sample_rate: f32, duration_secs: f32) -> Vec<f32, 1000> {
        let mut wave = Vec::new();
        let num_samples = (sample_rate * duration_secs) as usize;
        for i in 0..num_samples {
            let t = i as f32 / sample_rate;
            let sample = (2.0 * core::f32::consts::PI * freq * t).sin();
            wave.push(sample).unwrap();
        }
        wave
    }

    /// Calculates the average power of a signal.
    fn signal_power(signal: &[f32]) -> f32 {
        signal.iter().map(|&s| s * s).sum::<f32>() / signal.len() as f32
    }

    #[test]
    fn test_notch_filter_attenuates_target_frequency() {
        let sample_rate = 1000.0;
        let notch_freq = 50.0;
        let other_freq = 10.0;

        let mut filter = IirNotchFilter::new(sample_rate, notch_freq, 5.0);

        // 1. Generate a sine wave at the notch frequency.
        let mut input_signal = generate_sine_wave(notch_freq, sample_rate, 1.0);
        let input_power = signal_power(&input_signal);

        // 2. Apply the filter.
        let mut output_signal = Vec::<f32, 1000>::new();
        for &sample in &input_signal {
            output_signal.push(filter.apply(sample)).unwrap();
        }
        let output_power = signal_power(&output_signal[50..]); // Ignore transient response at start

        // 3. Verify that the power is significantly reduced.
        assert!(output_power < input_power * 0.01, "Filter did not sufficiently attenuate target frequency.");

        // 4. Reset filter and test a non-notch frequency.
        filter.reset();
        input_signal = generate_sine_wave(other_freq, sample_rate, 1.0);
        let input_power_other = signal_power(&input_signal);

        output_signal.clear();
        for &sample in &input_signal {
            output_signal.push(filter.apply(sample)).unwrap();
        }
        let output_power_other = signal_power(&output_signal[50..]);

        // 5. Verify that the power of the other frequency is not significantly reduced.
        assert!(output_power_other > input_power_other * 0.9, "Filter incorrectly attenuated a non-target frequency.");
    }

    #[test]
    fn test_filter_on_mixed_signal() {
        let sample_rate = 1000.0;
        let notch_freq = 50.0;
        let pass_freq = 10.0;

        let mut filter = IirNotchFilter::new(sample_rate, notch_freq, 3.0);

        // Create a signal that is the sum of two sine waves.
        let mut mixed_signal = Vec::<f32, 1000>::new();
        let num_samples = (sample_rate * 1.0) as usize;
        for i in 0..num_samples {
            let t = i as f32 / sample_rate;
            let sample1 = (2.0 * core::f32::consts::PI * notch_freq * t).sin();
            let sample2 = (2.0 * core::f32::consts::PI * pass_freq * t).sin();
            mixed_signal.push(sample1 + sample2).unwrap();
        }

        let input_power = signal_power(&mixed_signal);

        // Apply the filter
        let mut output_signal = Vec::<f32, 1000>::new();
        for &sample in &mixed_signal {
            output_signal.push(filter.apply(sample)).unwrap();
        }
        let output_power = signal_power(&output_signal[50..]);

        // The output power should be roughly half, since one of the two components was removed.
        assert!(output_power < input_power * 0.6 && output_power > input_power * 0.4);
    }
}
