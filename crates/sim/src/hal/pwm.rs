use hal::pwm::Pwm;
use atomic_float::AtomicF32;
use std::sync::Arc;

#[derive(Clone)]
pub struct VirtualPwmChannel {
    duty_cycle: Arc<AtomicF32>,
}

impl VirtualPwmChannel {
    pub fn new() -> Self {
        Self {
            duty_cycle: Arc::new(AtomicF32::new(0.0)),
        }
    }

    pub fn get_duty_cycle(&self) -> f32 {
        self.duty_cycle.load(std::sync::atomic::Ordering::Relaxed)
    }
}

pub struct VirtualPwm {
    max_duty: u16,
}

impl VirtualPwm {
    pub fn new(max_duty: u16) -> Self {
        Self { max_duty }
    }
}

impl Pwm for VirtualPwm {
    type Error = ();
    type Channel = VirtualPwmChannel;

    async fn enable(&mut self, _channel: Self::Channel) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn disable(&mut self, channel: Self::Channel) -> Result<(), Self::Error> {
        channel.duty_cycle.store(0.0, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn set_duty_cycle(&mut self, channel: Self::Channel, duty: u16) -> Result<(), Self::Error> {
        let duty_float = duty as f32 / self.max_duty as f32;
        channel.duty_cycle.store(duty_float, std::sync::atomic::Ordering::Relaxed);
        Ok(())
    }

    async fn set_period(&mut self, _period_hz: u32) -> Result<(), Self::Error> {
        Ok(())
    }

    async fn get_max_duty(&self) -> Result<u16, Self::Error> {
        Ok(self.max_duty)
    }
}
