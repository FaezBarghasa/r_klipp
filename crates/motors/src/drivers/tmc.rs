use hal::uart::Uart;
use r_klipp_api::FaultCode;

pub struct Tmc2209<UART>
where
    UART: Uart,
{
    uart: UART,
}

impl<UART> Tmc2209<UART>
where
    UART: Uart,
{
    pub fn new(uart: UART) -> Self {
        Self { uart }
    }

    pub async fn read_register(&mut self, address: u8) -> Result<u32, ()> {
        let write_buf = [0x05, 0x00, address];
        self.uart.write(&write_buf).await.map_err(|_| ())?;

        let mut read_buf = [0u8; 8];
        self.uart.read(&mut read_buf).await.map_err(|_| ())?;

        Ok(u32::from_be_bytes([read_buf[4], read_buf[5], read_buf[6], read_buf[7]]))
    }

    pub async fn write_register(&mut self, address: u8, value: u32) -> Result<(), ()> {
        let value_bytes = value.to_be_bytes();
        let write_buf = [0x05, 0x00, 0x80 | address, value_bytes[0], value_bytes[1], value_bytes[2], value_bytes[3]];
        self.uart.write(&write_buf).await.map_err(|_| ())?;
        Ok(())
    }

    pub async fn poll_task(&mut self, fault_queue: embassy_sync::channel::Sender<'static, embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex, FaultCode, 1>) {
        loop {
            if let Ok(sg_result) = self.read_register(0x6F).await {
                if sg_result == 0 {
                    fault_queue.send(FaultCode::StallGuard).await;
                }
            }
            embassy_time::Timer::after(embassy_time::Duration::from_millis(100)).await;
        }
    }
}