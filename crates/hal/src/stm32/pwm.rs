use crate::pwm::Pwm as HalPwm;
use embassy_stm32::timer::simple_pwm::{SimplePwm, PwmPin};
use embassy_stm32::time::Hertz;
use embassy_stm32::timer::Channel;

pub struct Stm32Pwm<'d, T> {
    pwm: SimplePwm<'d, T>,
}

impl<'d, T> Stm32Pwm<'d, T>
where
    T: embassy_stm32::timer::Instance,
{
    pub fn new(
        tim: T,
        ch1: Option<PwmPin<'d, T, embassy_stm32::timer::Ch1>>,
        ch2: Option<PwmPin<'d, T, embassy_stm32::timer::Ch2>>,
        ch3: Option<PwmPin<'d, T, embassy_stm32::timer::Ch3>>,
        ch4: Option<PwmPin<'d, T, embassy_stm32::timer::Ch4>>,
        freq: Hertz,
    ) -> Self {
        Self {
            pwm: SimplePwm::new(tim, ch1, ch2, ch3, ch4, freq, Default::default()),
        }
    }
}

impl<'d, T> HalPwm for Stm32Pwm<'d, T>
where
    T: embassy_stm32::timer::Instance,
{
    type Error = ();
    type Channel = Channel;

    async fn enable(&mut self, channel: Self::Channel) -> Result<(), Self::Error> {
        self.pwm.enable(channel);
        Ok(())
    }

    async fn disable(&mut self, channel: Self::Channel) -> Result<(), Self::Error> {
        self.pwm.disable(channel);
        Ok(())
    }

    async fn set_duty_cycle(&mut self, channel: Self::Channel, duty: u16) -> Result<(), Self::Error> {
        self.pwm.set_duty(channel, duty);
        Ok(())
    }

    async fn set_period(&mut self, period_hz: u32) -> Result<(), Self::Error> {
        self.pwm.set_freq(Hertz(period_hz));
        Ok(())
    }

    async fn get_max_duty(&self) -> Result<u16, Self::Error> {
        Ok(self.pwm.get_max_duty())
    }
}
