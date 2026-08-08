use tokio::sync::mpsc;
use r_klipp_api::HostToMcu;
use crc8_rs::Crc8;

struct Frame {
    payload: Vec<u8>,
    crc: u8,
}

impl Frame {
    fn new(payload: Vec<u8>) -> Self {
        let mut crc8 = Crc8::new();
        crc8.update(&payload);
        Self {
            payload,
            crc: crc8.finish(),
        }
    }
}

pub async fn mcu_comms_actor(mut rx: mpsc::Receiver<HostToMcu>) {
    // In a real implementation, this would be a serial port
    let mut serial_writer = tokio::io::stdout();

    while let Some(cmd) = rx.recv().await {
        let mut buf = [0u8; 256];
        if let Ok(encoded) = postcard::to_slice(&cmd, &mut buf) {
            let frame = Frame::new(encoded.to_vec());
            // In a real implementation, we'd send frame.payload and frame.crc
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;

    #[tokio::test]
    async fn test_mcu_comms_actor() {
        let (tx, rx) = mpsc::channel(10);
        let actor_handle = tokio::spawn(mcu_comms_actor(rx));

        let waypoints = heapless::Vec::new();
        let cmd = HostToMcu::BasicTrajectory { waypoints, max_jerk: 10.0 };
        tx.send(cmd).await.unwrap();

        // In a real test, we'd need to mock the serial port and check the output.
        // For now, we just ensure the actor doesn't crash.
        actor_handle.await.unwrap();
    }
}
