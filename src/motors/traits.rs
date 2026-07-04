
pub enum MotorError {
    Fault,
    NotEnabled,
    // ...
}

pub trait MotorAxis {
    async fn command_position(&mut self, position: f32) -> Result<(), MotorError>;
    async fn read_actual_position(&self) -> Result<f32, MotorError>;
    async fn set_torque_limit(&mut self, limit: f32) -> Result<(), MotorError>;
    async fn enable(&mut self) -> Result<(), MotorError>;
    async fn disable(&mut self) -> Result<(), MotorError>;
    async fn fault_status(&self) -> Result<u32, MotorError>;
}

pub trait StepDirControl {
    // ...
}

pub trait AnalogVoltageControl {
    // ...
}

pub trait DigitalBusControl {
    // ...
}

pub trait DirectFocControl {
    // ...
}
