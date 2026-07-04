use hal::spi::SpiDevice;

pub struct Tmc2209<SPI>
where
    SPI: SpiDevice,
{
    spi: SPI,
}

impl<SPI> Tmc2209<SPI>
where
    SPI: SpiDevice,
{
    pub fn new(spi: SPI) -> Self {
        Self { spi }
    }

    pub async fn read_register(&mut self, address: u8) -> Result<u32, ()> {
        let mut read_buf = [0u8; 5];
        let write_buf = [address & 0x7F, 0, 0, 0, 0];
        self.spi.transfer(&mut read_buf, &write_buf).await.unwrap();
        Ok(u32::from_be_bytes([read_buf[1], read_buf[2], read_buf[3], read_buf[4]]))
    }

    pub async fn write_register(&mut self, address: u8, value: u32) -> Result<(), ()> {
        let value_bytes = value.to_be_bytes();
        let write_buf = [address | 0x80, value_bytes[0], value_bytes[1], value_bytes[2], value_bytes[3]];
        self.spi.write(&write_buf).await.unwrap();
        Ok(())
    }
}

#[embassy_executor::task]
pub async fn tmc_poll_task<SPI>(mut driver: Tmc2209<SPI>)
where
    SPI: SpiDevice + 'static,
{
    loop {
        // Poll status registers
        // let gstat = driver.read_register(0x01).await.unwrap();
        // if (gstat & 1) != 0 { // reset flag
        // }
        // if (gstat & 2) != 0 { // error flag
        // }

        embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;
    }
}
