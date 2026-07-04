pub trait Pwm {
    type Error;
    type Channel;

    async fn enable(&mut self, channel: Self::Channel) -> Result<(), Self::Error>;
    async fn disable(&mut self, channel: Self::Channel) -> Result<(), Self::Error>;
    async fn set_duty_cycle(&mut self, channel: Self::Channel, duty: u16) -> Result<(), Self::Error>;
    async fn set_period(&mut self, period_hz: u32) -> Result<(), Self::Error>;
    async fn get_max_duty(&self) -> Result<u16, Self::Error>;
}
