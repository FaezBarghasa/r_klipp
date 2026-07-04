use core::future::Future;

pub trait Timer {
    type Error;

    async fn delay_ms(&mut self, ms: u32) -> Result<(), Self::Error>;
    async fn delay_us(&mut self, us: u32) -> Result<(), Self::Error>;
}

pub trait Periodic: Timer {
    fn new_interval(
        &self,
        period_ms: u32,
    ) -> Result<impl Future<Output = Result<(), Self::Error>>, Self::Error>;
}
