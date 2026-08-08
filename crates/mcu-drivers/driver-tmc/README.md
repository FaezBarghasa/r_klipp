# Klipper in Rust: Trinamic Stepper Driver (TMC)

## Overview

This crate provides a generic driver for Trinamic stepper motor controllers (TMCs). It supports communication with the drivers over UART or SPI, allowing for advanced configuration and control.

## Features

*   **Register Access**: Provides a safe, high-level API for reading and writing the registers of the TMC drivers.
*   **Microstepping Control**: Allows for dynamic configuration of the microstepping mode.
*   **Current Control**: Supports setting the run and hold currents of the motor.
*   **StallGuard and CoolStep**: Includes support for Trinamic's advanced features like StallGuard for sensorless homing and CoolStep for power savings.
*   **Driver-Specific Support**: Has specialized support for common drivers like the TMC2209 and TMC2130.

## Usage

```rust
use driver_tmc::Tmc2209;
use embedded_hal::serial::{Read, Write};

fn setup_tmc2209<SERIAL>(serial: SERIAL) -> Tmc2209<SERIAL>
where
    SERIAL: Read<u8> + Write<u8>,
{
    let mut driver = Tmc2209::new(serial);
    driver.init().unwrap();
    driver.set_run_current(800); // 800mA
    driver.enable_coolstep(true);
    driver
}
```
