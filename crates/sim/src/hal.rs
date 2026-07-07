use async_trait::async_trait;
use r_klipp_api::hal::SerialLink;
use std::time::Duration;
use tokio::time::sleep;
use rand::Rng;

pub struct VirtualSerialLink {
    latency_ms: u64,
    packet_loss_percent: u8,
}

impl VirtualSerialLink {
    pub fn new(latency_ms: u64, packet_loss_percent: u8) -> Self {
        Self { latency_ms, packet_loss_percent }
    }
}

#[async_trait]
impl SerialLink for VirtualSerialLink {
    async fn send(&mut self, data: &[u8]) {
        sleep(Duration::from_millis(self.latency_ms / 2)).await;
        if rand::thread_rng().gen_range(0..100) >= self.packet_loss_percent {
            // In a real sim, this would write to a shared buffer
        }
    }

    async fn recv(&mut self, buf: &mut [u8]) -> usize {
        sleep(Duration::from_millis(self.latency_ms / 2)).await;
        if rand::thread_rng().gen_range(0..100) >= self.packet_loss_percent {
            // In a real sim, this would read from a shared buffer
            0
        } else {
            0
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_virtual_hal_link_simulator() {
        let mut link = VirtualSerialLink::new(50, 10);
        let start = std::time::Instant::now();
        link.send(&[]).await;
        let duration = start.elapsed();
        assert!(duration >= Duration::from_millis(25));
    }
}
