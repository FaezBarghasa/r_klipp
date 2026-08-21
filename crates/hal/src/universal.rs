//! Universal Hardware Abstraction Layer (HAL) Traits for multi-target machines.
//!
//! Decouples high-level motion planners, vision triggers, heaters, vacuum valves,
//! and spindles from physical MCU peripheral implementations.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HalError {
    PinError,
    AdcError,
    PwmError,
    BusError,
    NotSupported,
}

/// Universal Stepper Motor Axis Trait
pub trait StepperAxis {
    type Error;

    fn set_direction(&mut self, forward: bool) -> Result<(), Self::Error>;
    fn step_pulse(&mut self) -> Result<(), Self::Error>;
    fn set_enabled(&mut self, enabled: bool) -> Result<(), Self::Error>;
    fn set_current_ma(&mut self, current_ma: u16) -> Result<(), Self::Error>;
    fn set_microsteps(&mut self, microsteps: u16) -> Result<(), Self::Error>;
}

/// Universal Discrete Digital Output Trait
/// Used for 3D printer fans, PnP vacuum valves, camera shutters, and CNC coolant relays.
pub trait DigitalOutput {
    type Error;

    fn set_state(&mut self, active: bool) -> Result<(), Self::Error>;
    fn is_active(&self) -> Result<bool, Self::Error>;
    
    /// Schedules a pin state change at an exact future hardware timer tick timestamp (e.g. for vision synchronization).
    fn scheduled_set_state(&mut self, target_tick: u64, active: bool) -> Result<(), Self::Error> {
        let _ = target_tick;
        self.set_state(active)
    }
}

/// Universal Analog Input Trait
/// Used for 3D printer thermistors, PnP vacuum pressure sensors, and CNC load cells.
pub trait AnalogInput {
    type Error;

    fn read_raw(&mut self) -> Result<u16, Self::Error>;
    fn read_scaled(&mut self) -> Result<f32, Self::Error>;
}

/// Universal PWM Output Trait
/// Used for hotends/beds, PnP blower fans, and CNC VFD spindle speed control.
pub trait PwmOutput {
    type Error;

    fn set_duty_cycle(&mut self, duty: f32) -> Result<(), Self::Error>; // 0.0 to 1.0
    fn get_duty_cycle(&self) -> f32;
    fn set_frequency(&mut self, freq_hz: u32) -> Result<(), Self::Error>;
}

/// Universal Communication Bus Trait
/// Used for TMC driver SPI/UART, smart CAN-FD feeders, and Modbus RS485 spindles.
pub trait CommunicationBus {
    type Error;

    fn transmit(&mut self, data: &[u8]) -> Result<(), Self::Error>;
    fn receive(&mut self, buffer: &mut [u8]) -> Result<usize, Self::Error>;
}
