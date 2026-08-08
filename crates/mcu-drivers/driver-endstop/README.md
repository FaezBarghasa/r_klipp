# Klipper in Rust: Endstop Driver

## Overview

This crate provides a driver for reading the state of endstops, which are used for homing and as limit switches. It supports both simple GPIO-based endstops and more complex types like optical or hall-effect sensors.

## Features

*   **Debouncing**: Includes software debouncing to prevent spurious readings from mechanical switches.
*   **Active High/Low**: Can be configured for both active-high and active-low endstop configurations.
*   **Async API**: Provides an async `wait_for_trigger` function that can be used to wait for an endstop to be triggered without blocking the executor.

## Usage

```rust
use driver_endstop::Endstop;
use embedded_hal::digital::v2::InputPin;

fn setup_endstop<PIN: InputPin>(pin: PIN) -> Endstop<PIN> {
    // Endstop is active low, with a 50ms debounce time
    Endstop::new(pin, true, 50)
}
```
