use crate::traits::{MotorAxis, AnalogVoltageControl, MotorError};

pub struct VoiceCoil<DAC>
where
    DAC: AnalogVoltageControl,
{
    dac: DAC,
    gravity_compensation: f32,
}

impl<DAC> VoiceCoil<DAC>
where
    DAC: AnalogVoltageControl,
{
    pub fn new(dac: DAC, gravity_compensation: f32) -> Self {
        Self {
            dac,
            gravity_compensation,
        }
    }
}

impl<DAC> MotorAxis for VoiceCoil<DAC>
where
    DAC: AnalogVoltageControl,
{
    async fn command_position(&mut self, position: f32) -> Result<(), MotorError> {
        // Voice coils are typically current controlled, which is proportional to voltage.
        // A real implementation would have a PID loop here.
        let voltage = position * 0.1 + self.gravity_compensation;
        self.dac.set_voltage(voltage).await;
        Ok(())
    }

    async fn read_actual_position(&mut self) -> Result<f32, MotorError> {
        // Position would be read from an encoder, which is not part of this trait.
        Err(MotorError::Fault)
    }

    async fn set_torque_limit(&mut self, limit: f32) -> Result<(), MotorError> {
        // Torque is proportional to current, which is proportional to voltage.
        self.dac.set_voltage(limit).await;
        Ok(())
    }

    async fn enable(&mut self) -> Result<(), MotorError> {
        Ok(())
    }

    async fn disable(&mut self) -> Result<(), MotorError> {
        self.dac.set_voltage(0.0).await;
        Ok(())
    }

    async fn fault_status(&mut self) -> Result<u32, MotorError> {
        Ok(0)
    }
}
