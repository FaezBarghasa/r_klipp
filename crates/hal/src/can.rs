pub struct Frame {
    pub id: u32,
    pub data: [u8; 8],
    pub len: u8,
}

pub trait Can {
    type Error;

    async fn transmit(&mut self, frame: &Frame) -> Result<(), Self::Error>;
    async fn receive(&mut self) -> Result<Frame, Self::Error>;
}
