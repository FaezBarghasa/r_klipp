use hal::dma::Dma;

pub struct VirtualDma;

impl Dma for VirtualDma {
    type Error = ();

    async fn transfer_from_peripheral_to_memory<P, M>(
        &mut self,
        _peripheral_address: P,
        _memory_address: M,
        _len: usize,
    ) -> Result<(), Self::Error>
    where
        P: Into<u32>,
        M: Into<u32>,
    {
        // In a real simulation, this would involve some memory manipulation.
        Ok(())
    }

    async fn transfer_from_memory_to_peripheral<M, P>(
        &mut self,
        _memory_address: M,
        _peripheral_address: P,
        _len: usize,
    ) -> Result<(), Self::Error>
    where
        M: Into<u32>,
        P: Into<u32>,
    {
        Ok(())
    }
}
