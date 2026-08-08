use hal::adc::Adc;
use atomic_float::AtomicF32;
use std::sync::Arc;

#[derive(Clone)]
pub struct VirtualAdcChannel {
    value: Arc<AtomicF32>,
}

impl VirtualAdcChannel {
    pub fn new(initial_value: f32) -> Self {
        Self {
            value: Arc::new(AtomicF32::new(initial_value)),
        }
    }

    pub fn set_value(&self, value: f32) {
        self.value.store(value, std::sync::atomic::Ordering::Relaxed);
    }
}

pub struct VirtualAdc;

impl Adc<u16> for VirtualAdc {
    type Error = ();
    type Channel = VirtualAdcChannel;

    async fn read(&mut self, channel: &mut Self::Channel) -> Result<u16, Self::Error> {
        // Scale the f32 value to a u16
        let value = channel.value.load(std::sync::atomic::Ordering::Relaxed);
        Ok((value * 4095.0 / 3.3) as u16)
    }
}
