use crate::timer::Timer;
use embassy_time::{Duration, Timer as EmbassyTimer};

pub struct Stm32Timer;

impl Timer for Stm32Timer {
    type Error = ();

    async fn delay_ms(&mut self, ms: u32) -> Result<(), Self::Error> {
        EmbassyTimer::after(Duration::from_millis(ms as u64)).await;
        Ok(())
    }

    async fn delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        EmbassyTimer::after(Duration::from_micros(us as u64)).await;
        Ok(())
    }
}
