
use embedded_hal_async::spi;

pub trait DmaStepEngine {
    async fn stream_steps(&mut self, buffer: &[u32]);
    fn swap_buffer(&mut self);
}

pub trait ThermalAdc {
    async fn read_temperatures(&mut self) -> [u16; 4];
}

pub trait InternalFlash {
    async fn erase_bank(&mut self, bank: u8);
    async fn write_page(&mut self, page: u32, data: &[u8]);
}
