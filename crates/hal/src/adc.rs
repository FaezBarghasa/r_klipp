pub trait Adc<WORD> {
    type Error;
    type Channel;

    async fn read(&mut self, channel: &mut Self::Channel) -> Result<WORD, Self::Error>;
}
