
use async_trait::async_trait;

#[async_trait]
pub trait DmaStepEngine {
    async fn stream_steps(&mut self, buffer: &[u32]);
    fn swap_buffer(&mut self);
}

#[async_trait]
pub trait ThermalAdc {
    async fn read_temperatures(&mut self) -> [u16; 4];
}

#[async_trait]
pub trait InternalFlash {
    async fn erase_bank(&mut self, bank: u8);
    async fn write_page(&mut self, page: u32, data: &[u8]);
}

#[async_trait]
pub trait SerialLink {
    async fn send(&mut self, data: &[u8]);
    async fn recv(&mut self, buf: &mut [u8]) -> usize;
}
