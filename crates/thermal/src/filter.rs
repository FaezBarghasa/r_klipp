//! ADC Signal Filtering
//!
//! Provides simple filters to reduce noise from ADC readings before they are
//! used in other parts of the system, like the PID controller.

use num_traits::{Float, FromPrimitive};

/// A generic trait for signal filters.
pub trait Filter<T> {
    /// Adds a new sample to the filter.
    fn add_sample(&mut self, sample: T);
    /// Returns the current filtered output value.
    fn output(&self) -> T;
}

/// A simple first-order low-pass filter.
///
/// This filter smooths out a signal by giving more weight to recent samples.
/// The `alpha` value determines the responsiveness vs. smoothness trade-off.
/// `output = alpha * new_sample + (1 - alpha) * old_output`
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LowPassFilter<T: Float> {
    alpha: T,
    last_output: T,
}

impl<T: Float> LowPassFilter<T> {
    /// Creates a new `LowPassFilter`.
    ///
    /// # Arguments
    /// * `alpha` - The smoothing factor, between 0.0 and 1.0.
    ///   - A smaller alpha results in more smoothing but slower response.
    ///   - A larger alpha results in less smoothing but faster response.
    /// * `initial_value` - The initial output value of the filter.
    pub fn new(alpha: T, initial_value: T) -> Self {
        assert!(alpha >= T::zero() && alpha <= T::one());
        Self {
            alpha,
            last_output: initial_value,
        }
    }
}

impl<T: Float> Filter<T> for LowPassFilter<T> {
    fn add_sample(&mut self, sample: T) {
        self.last_output = self.alpha * sample + (T::one() - self.alpha) * self.last_output;
    }

    fn output(&self) -> T {
        self.last_output
    }
}

/// A moving average filter.
///
/// This filter calculates the average of the last `N` samples. It's effective
/// at smoothing out random noise but can be slow to respond to actual changes.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MovingAverageFilter<T, const N: usize>
where
    T: FromPrimitive + Copy + Default + core::ops::Add<Output = T> + core::ops::Sub<Output = T> + core::ops::Div<Output = T>,
{
    samples: [T; N],
    index: usize,
    sum: T,
    count: usize,
}

impl<T, const N: usize> MovingAverageFilter<T, N>
where
    T: FromPrimitive + Copy + Default + core::ops::Add<Output = T> + core::ops::Sub<Output = T> + core::ops::Div<Output = T>,
{
    /// Creates a new `MovingAverageFilter`.
    /// The filter is initialized with default values (usually zero).
    pub fn new() -> Self {
        assert!(N > 0, "Window size must be greater than zero.");
        Self {
            samples: [T::default(); N],
            index: 0,
            sum: T::default(),
            count: 0,
        }
    }
}

impl<T, const N: usize> Default for MovingAverageFilter<T, N>
where
    T: FromPrimitive + Copy + Default + core::ops::Add<Output = T> + core::ops::Sub<Output = T> + core::ops::Div<Output = T>,
{
    fn default() -> Self {
        Self::new()
    }
}


impl<T, const N: usize> Filter<T> for MovingAverageFilter<T, N>
where
    T: FromPrimitive + Copy + Default + core::ops::Add<Output = T> + core::ops::Sub<Output = T> + core::ops::Div<Output = T>,
{
    fn add_sample(&mut self, sample: T) {
        // Subtract the oldest sample from the sum
        self.sum = self.sum - self.samples[self.index];
        // Add the new sample to the sum
        self.sum = self.sum + sample;
        // Store the new sample
        self.samples[self.index] = sample;
        // Move to the next index
        self.index = (self.index + 1) % N;

        if self.count < N {
            self.count += 1;
        }
    }

    fn output(&self) -> T {
        if self.count == 0 {
            T::default()
        } else {
            self.sum / T::from_usize(self.count).unwrap()
        }
    }
}
