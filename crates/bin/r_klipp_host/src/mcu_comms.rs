use tokio::sync::mpsc::{Receiver, Sender};
use tokio_serial::{SerialPortBuilderExt, SerialStream};
use r_klipp_api::{HostToMcu, McuToHost, Frame};
use postcard::to_vec;
use heapless::Vec;

pub struct McuComms {
    tx: Sender<HostToMcu>,
    rx: Receiver<McuToHost>,
    serial: SerialStream,
}

impl McuComms {
    pub async fn new(port: &str, baud_rate: u32) -> Result<Self, tokio_serial::Error> {
        let serial = tokio_serial::new(port, baud_rate).open_native_async()?;
        let (tx, mut host_rx) = tokio::sync::mpsc::channel(100);
        let (mcu_tx, rx) = tokio::sync::mpsc::channel(100);

        let mut actor = McuCommsActor {
            host_rx,
            mcu_tx,
            serial,
        };

        tokio::spawn(async move {
            actor.run().await;
        });

        Ok(Self { tx, rx, serial: todo!() })
    }

    pub fn get_sender(&self) -> Sender<HostToMcu> {
        self.tx.clone()
    }

    pub async fn recv(&mut self) -> Option<McuToHost> {
        self.rx.recv().await
    }
}

struct McuCommsActor {
    host_rx: Receiver<HostToMcu>,
    mcu_tx: Sender<McuToHost>,
    serial: SerialStream,
}

impl McuCommsActor {
    async fn run(&mut self) {
        loop {
            tokio::select! {
                Some(cmd) = self.host_rx.recv() => {
                    let frame = Frame::new(cmd).unwrap();
                    let bytes = frame.to_bytes().unwrap();
                    let mut outgoing_buffer: Vec<u8, 1024> = Vec::new();
                    outgoing_buffer.extend_from_slice(&bytes).unwrap();

                    if outgoing_buffer.len() > 1000 { // Simplified flow control
                        // In a real implementation, we'd wait for the MCU buffer to clear
                        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                    }

                    self.serial.writable().await.unwrap();
                    self.serial.try_write(&outgoing_buffer).unwrap();
                }
                // Add a branch here to read from self.serial and send to self.mcu_tx
            }
        }
    }
}
