pub trait SpiDevice {
    type Error;

    async fn write(&mut self, bytes: &[u8]) -> Result<(), Self::Error>;
    async fn read(&mut self, bytes: &mut [u8]) -> Result<(), Self::Error>;
    async fn transfer(&mut self, read: &mut [u8], write: &[u8]) -> Result<(), Self::Error>;
    async fn transfer_in_place(&mut self, bytes: &mut [u8]) -> Result<(), Self::Error>;
}
