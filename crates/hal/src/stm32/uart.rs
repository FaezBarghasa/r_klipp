use crate::uart::Uart as HalUart;
use embassy_stm32::usart::{Uart as EmbassyUart, Config};

pub struct Stm32Uart<'d, T> {
    uart: EmbassyUart<'d, T, embassy_stm32::usart::TxDma<T>, embassy_stm32::usart::RxDma<T>>,
}

impl<'d, T> Stm32Uart<'d, T>
where
    T: embassy_stm32::usart::Instance,
{
    pub fn new(
        uart: T,
        rx: impl embassy_stm32::usart::RxPin<T>,
        tx: impl embassy_stm32::usart::TxPin<T>,
        tx_dma: impl embassy_stm32::usart::TxDma<T>,
        rx_dma: impl embassy_stm32::usart::RxDma<T>,
    ) -> Self {
        Self {
            uart: EmbassyUart::new(uart, rx, tx, tx_dma, rx_dma, Config::default()).unwrap(),
        }
    }
}

impl<'d, T> HalUart for Stm32Uart<'d, T>
where
    T: embassy_stm32::usart::Instance,
{
    type Error = embassy_stm32::usart::Error;

    async fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        self.uart.write(bytes).await
    }

    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        self.uart.read(buffer).await.map_err(|e| e.into())
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        self.uart.flush().await
    }
}
