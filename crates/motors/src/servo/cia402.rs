use hal::can::{Can, Frame};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Cia402State {
    SwitchOnDisabled,
    ReadyToSwitchOn,
    SwitchedOn,
    OperationEnabled,
    Fault,
}

pub struct Cia402Proxy<CAN>
where
    CAN: Can,
{
    can: CAN,
    node_id: u8,
    state: Cia402State,
}

impl<CAN> Cia402Proxy<CAN>
where
    CAN: Can,
{
    pub fn new(can: CAN, node_id: u8) -> Self {
        Self {
            can,
            node_id,
            state: Cia402State::SwitchOnDisabled,
        }
    }

    pub async fn transition_to(&mut self, target_state: Cia402State) -> Result<(), ()> {
        // This is a simplified state machine. A real implementation would be more robust.
        match (self.state, target_state) {
            (Cia402State::SwitchOnDisabled, Cia402State::ReadyToSwitchOn) => {
                // Send Shutdown command
                self.send_control_word(0x06).await?;
                self.state = Cia402State::ReadyToSwitchOn;
            }
            (Cia402State::ReadyToSwitchOn, Cia402State::SwitchedOn) => {
                // Send Switch On command
                self.send_control_word(0x07).await?;
                self.state = Cia402State::SwitchedOn;
            }
            (Cia402State::SwitchedOn, Cia402State::OperationEnabled) => {
                // Send Enable Operation command
                self.send_control_word(0x0F).await?;
                self.state = Cia402State::OperationEnabled;
            }
            _ => return Err(()),
        }
        Ok(())
    }

    async fn send_control_word(&mut self, control_word: u16) -> Result<(), ()> {
        let bytes = control_word.to_le_bytes();
        let frame = Frame {
            id: 0x600 + self.node_id as u32,
            data: [0x40, 0x60, 0x00, bytes[0], bytes[1], 0, 0, 0], // SDO write to 0x6040
            len: 8,
        };
        self.can.transmit(&frame).await.map_err(|_| ())
    }

    pub async fn set_profile_position(&mut self, position: i32) -> Result<(), ()> {
        let bytes = position.to_le_bytes();
        let frame = Frame {
            id: 0x600 + self.node_id as u32,
            data: [0x7A, 0x60, 0x00, bytes[0], bytes[1], bytes[2], bytes[3], 0], // SDO write to 0x607A
            len: 8,
        };
        self.can.transmit(&frame).await.map_err(|_| ())
    }
}

#[embassy_executor::task]
pub async fn cia402_proxy_task<CAN>(mut proxy: Cia402Proxy<CAN>)
where
    CAN: Can + 'static,
{
    loop {
        // This task would receive commands from the motion planner and send them to the drive.
        embassy_time::Timer::after(embassy_time::Duration::from_millis(10)).await;
    }
}
