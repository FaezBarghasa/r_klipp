# `r_klipp_hal`: Universal Hardware Abstraction Layer

`r_klipp_hal` provides domain-agnostic, `no_std` Rust hardware abstraction traits designed to run identically on bare-metal microcontrollers (STM32, RP2040, ESP32) and host simulators.

---

## 🎯 Universal Traits

Defined in [`crates/hal/src/universal.rs`](src/universal.rs):

### 1. `StepperAxis`
Abstracts motor step generation, direction selection, and enable/disable operations.
```rust
pub trait StepperAxis {
    fn set_direction(&mut self, forward: bool) -> Result<(), HalError>;
    fn step(&mut self) -> Result<(), HalError>;
    fn enable(&mut self, enabled: bool) -> Result<(), HalError>;
    fn is_enabled(&self) -> bool;
}
```

### 2. `DigitalOutput`
Used interchangeably for cooling fans (3D printers), vacuum solenoids (Pick & Place), or coolant pumps (CNC). Supports scheduled discrete microsecond-accurate state changes:
```rust
pub trait DigitalOutput {
    fn set_state(&mut self, high: bool) -> Result<(), HalError>;
    fn scheduled_set_state(&mut self, high: bool, timestamp_us: u64) -> Result<(), HalError>;
}
```

### 3. `AnalogInput`
Read raw ADC values and convert to millivolts for thermistors, load cells, or vacuum pressure sensors.

### 4. `PwmOutput`
Configures PWM frequency and variable duty cycle for heaters, fans, and VFD spindle speed outputs.

### 5. `CommunicationBus`
Provides non-blocking UART, SPI, and CAN-FD communication primitives.
