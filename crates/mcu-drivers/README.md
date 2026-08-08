# Klipper in Rust: MCU and Peripheral Drivers Crate

## Overview

The `mcu-drivers` crate provides a Hardware Abstraction Layer (HAL) and a collection of drivers for various MCU peripherals and external components commonly found in 3D printers.

This crate is a `no_std` library, designed to be portable across different MCU architectures.

## Structure

This crate is organized as a collection of sub-crates and modules, each providing a driver for a specific piece of hardware.

*   **HAL Implementation**: Contains the implementation of the `embedded-hal` traits for our supported MCUs (e.g., `stm32f4xx-hal`).
*   **Stepper Drivers**: Drivers for common stepper motor drivers, such as:
    *   `driver-tmc`: A generic driver for Trinamic stepper motors, supporting communication over SPI or UART.
    *   `driver-drv8825`: A driver for the DRV8825 stepper motor controller.
*   **Sensor Drivers**:
    *   `driver-thermistor`: A driver for reading thermistors and converting ADC values to temperature.
    *   `driver-endstop`: A driver for reading the state of mechanical or optical endstops.
*   **Other Peripherals**: Drivers for other components like fans, heaters, and displays.

## Design Philosophy

*   **`embedded-hal` Compliance**: All drivers are written against the traits defined in the `embedded-hal` crate. This makes them generic and reusable across any MCU that has an `embedded-hal` implementation.
*   **Asynchronous API**: Where appropriate, drivers expose an asynchronous API that is compatible with the `embassy` framework. This allows for non-blocking access to peripherals.
*   **Feature-Gated**: Each driver and HAL implementation is enabled via a Cargo feature flag, allowing users to compile only the code they need, which is essential for resource-constrained MCUs.

## Usage

To use a driver from this crate, enable the corresponding feature in your `Cargo.toml` and initialize the driver with the appropriate peripheral from your HAL.

```rust
// Example of initializing a TMC2209 stepper driver
use mcu_drivers::driver_tmc::Tmc2209;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::serial::{Config, Serial};

fn setup_stepper_driver() {
    let dp = stm32f4xx_hal::pac::Peripherals::take().unwrap();
    let rcc = dp.RCC.constrain();
    let clocks = rcc.cfgr.freeze();
    let gpioa = dp.GPIOA.split();

    let tx = gpioa.pa2.into_alternate_af7();
    let rx = gpioa.pa3.into_alternate_af7();
    let serial = Serial::new(
        dp.USART2,
        (tx, rx),
        Config::default().baudrate(115200.bps()),
        &clocks,
    )
    .unwrap();

    let mut driver = Tmc2209::new(serial);
    driver.set_run_current(800); // Set run current to 800mA
}
```
