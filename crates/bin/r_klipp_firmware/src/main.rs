#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use r_klipp_api::{HostToMcu, McuToHost, LinkHealth};
use r_klipp_api::hal::SerialLink; // Assuming a mock implementation for now

struct MockSerialLink;

#[async_trait::async_trait]
impl SerialLink for MockSerialLink {
    async fn send(&mut self, data: &[u8]) {
        // Mock send
    }
    async fn recv(&mut self, buf: &mut [u8]) -> usize {
        // Mock receive
        0
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawner.spawn(link_monitor(MockSerialLink)).unwrap();
}

#[embassy_executor::task]
async fn link_monitor(mut serial: impl SerialLink + 'static) {
    let mut last_sync_time = 0;
    let mut rtt_us = 0;

    loop {
        let mut buf = [0u8; 128];
        if let Ok(len) = embassy_time::with_timeout(Duration::from_millis(1), serial.recv(&mut buf)).await {
            if len > 0 {
                if let Ok(msg) = postcard::from_bytes::<HostToMcu>(&buf[..len]) {
                    if let HostToMcu::SyncClock(host_time) = msg {
                        last_sync_time = embassy_time::Instant::now().as_micros() as u64;
                        rtt_us = (last_sync_time - host_time) as u32;
                    }
                }
            }
        }

        let link_health = LinkHealth {
            rtt_us,
            buffer_fill_percent: 50, // Mock value
            dropped_packets: 0, // Mock value
        };

        let telemetry = McuToHost::Telemetry {
            pos: [0.0; 6], // Mock
            temps: [0.0; 4], // Mock
            link_health,
        };

        let mut send_buf = [0u8; 128];
        if let Ok(encoded) = postcard::to_slice(&telemetry, &mut send_buf) {
            serial.send(encoded).await;
        }

        Timer::after(Duration::from_millis(100)).await;
    }
}
