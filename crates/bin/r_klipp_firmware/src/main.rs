#![no_std]
#![no_main]

use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use r_klipp_api::{HostToMcu, McuToHost, LinkHealth};
pub trait SerialLink {
    async fn send(&mut self, data: &[u8]);
    async fn recv(&mut self, buf: &mut [u8]) -> usize;
}

struct MockSerialLink;

impl SerialLink for MockSerialLink {
    async fn send(&mut self, _data: &[u8]) {
        // Mock send
    }
    async fn recv(&mut self, _buf: &mut [u8]) -> usize {
        // Mock receive
        0
    }
}

#[embassy_executor::main]
async fn main(spawner: Spawner) {
    spawner.spawn(link_monitor(MockSerialLink).expect("Failed to create task"));
}

#[embassy_executor::task]
async fn link_monitor(mut serial: MockSerialLink) {
    let mut last_sync_time = 0;
    let mut rtt_us = 0;

    loop {
        let mut buf = [0u8; 128];
        if let Ok(len) = embassy_time::with_timeout(Duration::from_millis(1), serial.recv(&mut buf)).await {
            if len > 0 {
                if let Ok(msg) = postcard::from_bytes::<HostToMcu>(&buf[..len]) {
                    if let r_klipp_api::HostCommand::SyncClock { host_timestamp_us } = msg.command {
                        last_sync_time = embassy_time::Instant::now().as_micros() as u64;
                        rtt_us = (last_sync_time.saturating_sub(host_timestamp_us)) as u32;
                    }
                }
            }
        }

        let link_health = LinkHealth {
            rtt_us,
            buffer_fill_percent: 50, // Mock value
            dropped_packets: 0, // Mock value
        };

        let telemetry = McuToHost::new(r_klipp_api::McuPayload::Telemetry {
            pos: [0.0; 6], // Mock
            temps: [0.0; 4], // Mock
            link_health,
        });

        let mut send_buf = [0u8; 128];
        if let Ok(encoded) = postcard::to_slice(&telemetry, &mut send_buf) {
            serial.send(encoded).await;
        }

        Timer::after(Duration::from_millis(100)).await;
    }
}
