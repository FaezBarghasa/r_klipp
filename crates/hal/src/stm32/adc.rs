use crate::adc::Adc as HalAdc;
use embassy_stm32::adc::{Adc as EmbassyAdc, AnyAdc, AdcPin};

pub struct Stm32Adc<'d> {
    adc: EmbassyAdc<'d, AnyAdc>,
}

impl<'d> Stm32Adc<'d> {
    pub fn new(adc: AnyAdc) -> Self {
        let mut adc = EmbassyAdc::new(adc, Default::default());
        adc.enable();
        adc
    }
}

impl<'d, PIN> HalAdc<u16> for Stm32Adc<'d>
where
    PIN: AdcPin<AnyAdc>,
{
    type Error = ();
    type Channel = PIN;

    async fn read(&mut self, channel: &mut Self::Channel) -> Result<u16, Self::Error> {
        Ok(self.adc.read(channel))
    }
}
