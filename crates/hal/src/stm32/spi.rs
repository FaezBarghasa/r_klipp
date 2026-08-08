use crate::spi::SpiDevice as HalSpiDevice;
use embassy_stm32::spi::{Spi, Config};

pub struct Stm32Spi<'d, T> {
    spi: Spi<'d, T, embassy_stm32::spi::TxDma<T>, embassy_stm32::spi::RxDma<T>>,
}

impl<'d, T> Stm32Spi<'d, T>
where
    T: embassy_stm32::spi::Instance,
{
    pub fn new(
        spi: T,
        sck: impl embassy_stm32::spi::SckPin<T>,
        mosi: impl embassy_stm32::spi::MosiPin<T>,
        miso: impl embassy_stm32::spi::MisoPin<T>,
        tx_dma: impl embassy_stm32::spi::TxDma<T>,
        rx_dma: impl embassy_stm32::spi::RxDma<T>,
        config: Config,
    ) -> Self {
        Self {
            spi: Spi::new(spi, sck, mosi, miso, tx_dma, rx_dma, config),
        }
    }
}

impl<'d, T> HalSpiDevice for Stm32Spi<'d, T>
where
    T: embassy_stm32::spi::Instance,
{
    type Error = embassy_stm32::spi::Error;

    async fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.spi.write(bytes).await
    }

    async fn read(&mut self, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.spi.read(bytes).await
    }

    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error> {
        self.spi.transfer(read, write).await
    }

    async fn transfer_in_place(&mut self, bytes: &mut [u8]) -> Result<(), Self::Error> {
        self.spi.transfer_in_place(bytes).await
    }
}