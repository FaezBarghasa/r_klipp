pub trait Filter {
    type Item;
    fn add_sample(&mut self, sample: Self::Item);
    fn output(&self) -> Self::Item;
}

pub struct LowPassFilter {
    alpha: f64,
    val: f64,
}

impl LowPassFilter {
    pub fn new(alpha: f64, initial: f64) -> Self {
        Self { alpha, val: initial }
    }
}

impl Filter for LowPassFilter {
    type Item = f64;
    fn add_sample(&mut self, sample: f64) {
        self.val = self.alpha * sample + (1.0 - self.alpha) * self.val;
    }
    fn output(&self) -> f64 {
        self.val
    }
}

pub struct MovingAverageFilter<T, const N: usize> {
    buffer: [T; N],
    index: usize,
    count: usize,
}

impl<const N: usize> MovingAverageFilter<f32, N> {
    pub fn new() -> Self {
        Self {
            buffer: [0.0; N],
            index: 0,
            count: 0,
        }
    }
}

impl<const N: usize> Filter for MovingAverageFilter<f32, N> {
    type Item = f32;
    fn add_sample(&mut self, sample: f32) {
        self.buffer[self.index] = sample;
        self.index = (self.index + 1) % N;
        if self.count < N {
            self.count += 1;
        }
    }
    fn output(&self) -> f32 {
        if self.count == 0 {
            return 0.0;
        }
        let sum: f32 = self.buffer[..self.count].iter().sum();
        sum / (self.count as f32)
    }
}

pub struct ExponentialMovingAverage {
    pub alpha: f32,
}