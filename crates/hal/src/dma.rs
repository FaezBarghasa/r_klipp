pub trait Dma {
    type Error;

    async fn transfer_from_peripheral_to_memory<P, M>(
        &mut self,
        peripheral_address: P,
        memory_address: M,
        len: usize,
    ) -> Result<(), Self::Error>
    where
        P: Into<u32>,
        M: Into<u32>;

    async fn transfer_from_memory_to_peripheral<M, P>(
        &mut self,
        memory_address: M,
        peripheral_address: P,
        len: usize,
    ) -> Result<(), Self::Error>
    where
        M: Into<u32>,
        P: Into<u32>;
}
