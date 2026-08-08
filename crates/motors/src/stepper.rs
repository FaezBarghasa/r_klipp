use crate::traits::{MotorAxis, StepDirControl, MotorError};
use hal::gpio::{OutputPin, PinState};
use hal::timer::Timer;

pub struct StepperMotor<DIR, STEP, TIMER>
where
    DIR: OutputPin,
    STEP: OutputPin,
    TIMER: Timer,
{
    dir_pin: DIR,
    step_pin: STEP,
    timer: TIMER,
    position: f32,
    steps_per_mm: f32,
}

impl<DIR, STEP, TIMER> StepperMotor<DIR, STEP, TIMER>
where
    DIR: OutputPin,
    STEP: OutputPin,
    TIMER: Timer,
{
    pub fn new(dir_pin: DIR, step_pin: STEP, timer: TIMER, steps_per_mm: f32) -> Self {
        Self {
            dir_pin,
            step_pin,
            timer,
            position: 0.0,
            steps_per_mm,
        }
    }
}

impl<DIR, STEP, TIMER> MotorAxis for StepperMotor<DIR, STEP, TIMER>
where
    DIR: OutputPin,
    STEP: OutputPin,
    TIMER: Timer,
{
    async fn command_position(&mut self, position: f32) -> Result<(), MotorError> {
        let steps_to_move = (position - self.position) * self.steps_per_mm;
        let forward = steps_to_move > 0.0;
        self.dir_pin.set_state(if forward { PinState::High } else { PinState::Low }).unwrap();

        for _ in 0..steps_to_move.abs() as u32 {
            self.step_pin.set_high().unwrap();
            self.timer.delay_us(1).await.unwrap();
            self.step_pin.set_low().unwrap();
            self.timer.delay_us(1).await.unwrap();
        }
        self.position = position;
        Ok(())
    }

    async fn read_actual_position(&mut self) -> Result<f32, MotorError> {
        Ok(self.position)
    }

    async fn set_torque_limit(&mut self, limit: f32) -> Result<(), MotorError> {
        // Not supported by simple step/dir
        Err(MotorError::Fault)
    }

    async fn enable(&mut self) -> Result<(), MotorError> {
        // Assuming an enable pin would be handled here
        Ok(())
    }

    async fn disable(&mut self) -> Result<(), MotorError> {
        Ok(())
    }

    async fn fault_status(&mut self) -> Result<u32, MotorError> {
        Ok(0)
    }
}
