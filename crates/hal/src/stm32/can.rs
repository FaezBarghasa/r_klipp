use crate::can::Can as HalCan;
use crate::can::Frame as HalFrame;
use embassy_stm32::can::{Can as EmbassyCan, Frame as EmbassyFrame, StandardId};

pub struct Stm32Can<'d, T> {
    can: EmbassyCan<'d, T>,
}

impl<'d, T> Stm32Can<'d, T>
where
    T: embassy_stm32::can::Instance,
{
    pub fn new(can: T, rx: impl embassy_stm32::can::RxPin<T>, tx: impl embassy_stm32::can::TxPin<T>) -> Self {
        let can = EmbassyCan::new(can, rx, tx, Default::default());
        // can.set_bitrate(1_000_000);
        // can.enable();
        Self { can }
    }
}

impl<'d, T> HalCan for Stm32Can<'d, T>
where
    T: embassy_stm32::can::Instance,
{
    type Error = ();

    async fn transmit(&mut self, frame: &HalFrame) -> Result<(), Self::Error> {
        let id = StandardId::new(frame.id as u16).unwrap();
        let embassy_frame = EmbassyFrame::new_data(id, &frame.data[..frame.len as usize]);
        self.can.write(&embassy_frame).await;
        Ok(())
    }

    async fn receive(&mut self) -> Result<HalFrame, Self::Error> {
        let embassy_frame = self.can.read().await.unwrap();
        let mut data = [0u8; 8];
        data[..embassy_frame.data().len()].copy_from_slice(embassy_frame.data());
        Ok(HalFrame {
            id: embassy_frame.id().into(),
            data,
            len: embassy_frame.data().len() as u8,
        })
    }
}
