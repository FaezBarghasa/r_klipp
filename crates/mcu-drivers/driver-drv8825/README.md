# Klipper in Rust: DRV8825 Stepper Driver

## Overview

This crate provides a driver for the Texas Instruments DRV8825 stepper motor controller. The DRV8825 is a simple, step/direction-based driver.

## Features

*   **Step/Direction Control**: Provides basic control over the motor's position and direction.
*   **Microstepping**: Supports configuring the microstepping mode of the driver.
*   **Enable/Disable**: Allows for enabling and disabling the motor output.

## Usage

```rust
use driver_drv8825::DRV8825;
use embedded_hal::digital::v2::OutputPin;

fn setup_drv8825<STEP, DIR, ENABLE>(
    step_pin: STEP,
    dir_pin: DIR,
    enable_pin: ENABLE,
) -> DRV8825<STEP, DIR, ENABLE>
where
    STEP: OutputPin,
    DIR: OutputPin,
    ENABLE: OutputPin,
{
    let mut driver = DRV8825::new(step_pin, dir_pin, enable_pin);
    driver.enable();
    driver
}
```
