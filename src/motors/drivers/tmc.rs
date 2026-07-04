use crate::hal::traits::{Spi, Uart};

pub struct TmcDriver<C> {
    comms: C,
}

impl<C: Spi> TmcDriver<C> {
    pub async fn read_register(&mut self, reg: u8) -> Result<u32, ()> {
        let mut tx_buf = [reg & 0x7F, 0, 0, 0, 0];
        let mut rx_buf = [0; 5];
        self.comms.transfer(&mut tx_buf, &mut rx_buf).await.map_err(|_| ())?;
        Ok(u32::from_be_bytes([rx_buf[1], rx_buf[2], rx_buf[3], rx_buf[4]]))
    }

    pub async fn write_register(&mut self, reg: u8, value: u32) -> Result<(), ()> {
        let mut tx_buf = [reg | 0x80, 0, 0, 0, 0];
        tx_buf[1..].copy_from_slice(&value.to_be_bytes());
        self.comms.write(&tx_buf).await.map_err(|_| ())
    }
}

#[embassy_executor::task]
pub async fn tmc_background_task(mut driver: TmcDriver<impl Spi + 'static>) {
    loop {
        // Poll StallGuard and CoolStep registers
        if let Ok(gstat) = driver.read_register(0x01).await {
            // process status
        }
        embassy_time::Timer::after_millis(100).await;
    }
}
