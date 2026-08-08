use comms::Message;

pub struct ClockSyncActor {
    // Add state variables here
}

impl ClockSyncActor {
    pub fn new() -> Self {
        Self {}
    }

    pub async fn run(&mut self) {
        loop {
            // High-priority task for clock synchronization
            // This will likely involve CAN communication and a DPLL implementation.

            // For now, just yield
            embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;
        }
    }
}
