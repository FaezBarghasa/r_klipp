pub trait Adc<WORD> {
    type Error;
    type Channel;

    async fn read(&mut self, channel: &mut Self::Channel) -> Result<WORD, Self::Error>;
}

/// Continuous Circular DMA ADC Streamer
/// Continuously samples analog channels (e.g. thermistors, load-cell probes) into double-buffered DMA memory
/// without consuming main CPU execution cycles.
pub trait CircularDmaAdc<const CHANNELS: usize, const SAMPLES: usize> {
    type Error;

    /// Starts autonomous DMA sampling into hardware ring buffer
    fn start_continuous_sampling(&mut self) -> Result<(), Self::Error>;

    /// Reads the latest filtered channel average computed from DMA circular buffer
    fn get_filtered_channel(&self, channel_idx: usize) -> u16;

    /// Reads raw latest DMA sample for a specific channel
    fn get_raw_sample(&self, channel_idx: usize) -> u16;
}

/// Digital Exponential Moving Average (EMA) and Spike Rejection Filter for ADC channels
#[derive(Clone, Copy, Debug)]
pub struct DmaAdcFilter {
    pub filtered_value: f32,
    pub alpha: f32,
    pub max_spike_delta: f32,
}

impl DmaAdcFilter {
    pub const fn new(alpha: f32, max_spike_delta: f32) -> Self {
        Self {
            filtered_value: 0.0,
            alpha,
            max_spike_delta,
        }
    }

    /// Ingests a new raw ADC reading with noise spike rejection and exponential smoothing
    #[inline(always)]
    pub fn update(&mut self, raw_reading: u16) -> f32 {
        let raw = raw_reading as f32;
        if self.filtered_value == 0.0 {
            self.filtered_value = raw;
            return self.filtered_value;
        }

        // Spike rejection
        let delta = (raw - self.filtered_value).abs();
        let accepted_raw = if delta > self.max_spike_delta {
            if raw > self.filtered_value {
                self.filtered_value + self.max_spike_delta
            } else {
                self.filtered_value - self.max_spike_delta
            }
        } else {
            raw
        };

        // Exponential smoothing: y_k = alpha * x_k + (1 - alpha) * y_{k-1}
        self.filtered_value = self.alpha * accepted_raw + (1.0 - self.alpha) * self.filtered_value;
        self.filtered_value
    }
}

