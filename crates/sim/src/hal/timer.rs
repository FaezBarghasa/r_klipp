use hal::timer::Timer;
use std::time::Duration;
use tokio::time::sleep;

pub struct VirtualTimer;

impl Timer for VirtualTimer {
    type Error = ();

    async fn delay_ms(&mut self, ms: u32) -> Result<(), Self::Error> {
        sleep(Duration::from_millis(ms as u64)).await;
        Ok(())
    }

    async fn delay_us(&mut self, us: u32) -> Result<(), Self::Error> {
        sleep(Duration::from_micros(us as u64)).await;
        Ok(())
    }
}
