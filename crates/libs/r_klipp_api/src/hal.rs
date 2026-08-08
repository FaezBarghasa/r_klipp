
use async_trait::async_trait;

#[async_trait]
pub trait DmaStepEngine {
    async fn move_to(&mut self, steps: &[u32]);
    fn stop(&mut self);
}

#[async_trait]
pub trait ThermalAdc {
    async fn read_temp(&mut self) -> f32;
}

#[async_trait]
pub trait InternalFlash {
    async fn write(&mut self, address: u32, data: &[u8]);
    async fn read(&mut self, address: u32, length: usize) -> &[u8];
}

#[async_trait]
pub trait SerialLink {
    async fn send(&mut self, data: &[u8]);
    async fn recv(&mut self, buf: &mut [u8]) -> usize;
}
