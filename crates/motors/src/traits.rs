
pub enum MotorError {
    Fault,
    NotEnabled,
}

pub trait MotorAxis {
    async fn command_position(&mut self, position: f32) -> Result<(), MotorError>;
    async fn read_actual_position(&mut self) -> Result<f32, MotorError>;
    async fn set_torque_limit(&mut self, limit: f32) -> Result<(), MotorError>;
    async fn enable(&mut self) -> Result<(), MotorError>;
    async fn disable(&mut self) -> Result<(), MotorError>;
    async fn fault_status(&mut self) -> Result<u32, MotorError>;
}

pub trait StepDirControl {
    async fn set_direction(&mut self, forward: bool);
    async fn step(&mut self);
}

pub trait AnalogVoltageControl {
    async fn set_voltage(&mut self, voltage: f32);
}

pub trait DigitalBusControl {
    // This will be further defined by protocols like CiA 402
}

pub trait DirectFOCControl {
    async fn set_currents(&mut self, i_d: f32, i_q: f32);
}

pub trait DriverInterface: StepDirControl + AnalogVoltageControl + DigitalBusControl + DirectFOCControl {}
