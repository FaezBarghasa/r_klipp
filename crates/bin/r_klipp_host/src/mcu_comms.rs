use tokio::sync::mpsc;
use r_klipp_api::HostToMcu;

pub struct Frame {
    pub payload: Vec<u8>,
    pub crc: u8,
}

impl Frame {
    pub fn new(payload: Vec<u8>) -> Self {
        let crc = Self::compute_crc8(&payload);
        Self { payload, crc }
    }

    fn compute_crc8(data: &[u8]) -> u8 {
        let mut crc: u8 = 0x00;
        for &byte in data {
            crc ^= byte;
            for _ in 0..8 {
                if (crc & 0x80) != 0 {
                    crc = (crc << 1) ^ 0x07;
                } else {
                    crc <<= 1;
                }
            }
        }
        crc
    }
}

pub async fn mcu_comms_actor(mut rx: mpsc::Receiver<HostToMcu>) {
    while let Some(cmd) = rx.recv().await {
        let mut buf = [0u8; 256];
        if let Ok(encoded) = postcard::to_slice(&cmd, &mut buf) {
            let _frame = Frame::new(encoded.to_vec());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use r_klipp_api::HostCommand;

    #[tokio::test]
    async fn test_mcu_comms_actor() {
        let (tx, rx) = mpsc::channel(10);
        let actor_handle = tokio::spawn(mcu_comms_actor(rx));

        let waypoints = heapless::Vec::new();
        let cmd = HostToMcu::new(1, HostCommand::BasicTrajectory { waypoints, max_jerk: 10.0 });
        tx.send(cmd).await.unwrap();

        drop(tx);
        let _ = actor_handle.await;
    }
}
