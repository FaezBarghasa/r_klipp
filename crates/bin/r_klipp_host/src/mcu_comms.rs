use tokio::sync::mpsc;
use tokio_serial::SerialPortBuilderExt;
use r_klipp_api::{HostToMcu, Frame};
use postcard::to_slice;

pub async fn mcu_comm_actor(mut rx: mpsc::Receiver<HostToMcu>) {
    let mut port = tokio_serial::new("/dev/ttyACM0", 115200)
        .open_native_async()
        .unwrap();

    while let Some(coeffs) = rx.recv().await {
        let frame = Frame {
            magic: [0xDE, 0xAD, 0xBE, 0xEF],
            len: 0, // This would be calculated properly
            payload: coeffs,
            crc8: 0, // This would be calculated properly
        };

        let mut buffer = [0u8; 256];
        if let Ok(encoded) = to_slice(&frame, &mut buffer) {
            port.write_all(encoded).await.unwrap();
        }
    }
}
