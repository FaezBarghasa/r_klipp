use hal::uart::Uart;
use crossbeam_channel::{Receiver, Sender};

#[derive(Clone)]
pub struct VirtualUart {
    rx: Receiver<u8>,
    tx: Sender<u8>,
}

impl VirtualUart {
    pub fn new(rx: Receiver<u8>, tx: Sender<u8>) -> Self {
        Self { rx, tx }
    }
}

impl Uart for VirtualUart {
    type Error = ();

    async fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error> {
        for byte in bytes {
            self.tx.send(*byte).unwrap();
        }
        Ok(())
    }

    async fn read(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error> {
        let mut bytes_read = 0;
        for byte in buffer.iter_mut() {
            if let Ok(b) = self.rx.try_recv() {
                *byte = b;
                bytes_read += 1;
            } else {
                break;
            }
        }
        Ok(bytes_read)
    }

    async fn flush(&mut self) -> Result<(), Self::Error> {
        Ok(())
    }
}
