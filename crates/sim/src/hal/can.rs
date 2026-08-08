use hal::can::{Can, Frame};
use crossbeam_channel::{Receiver, Sender};

#[derive(Clone)]
pub struct VirtualCan {
    rx: Receiver<Frame>,
    tx: Sender<Frame>,
}

impl VirtualCan {
    pub fn new(rx: Receiver<Frame>, tx: Sender<Frame>) -> Self {
        Self { rx, tx }
    }
}

impl Can for VirtualCan {
    type Error = ();

    async fn transmit(&mut self, frame: &Frame) -> Result<(), Self::Error> {
        self.tx.send(frame.clone()).unwrap();
        Ok(())
    }

    async fn receive(&mut self) -> Result<Frame, Self::Error> {
        Ok(self.rx.recv().unwrap())
    }
}

// Need to implement Clone for Frame
impl Clone for Frame {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            data: self.data,
            len: self.len,
        }
    }
}
