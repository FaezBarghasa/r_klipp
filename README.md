# Klipper MCU Firmware in Rust

[![CI](https://github.com/your-username/klipper-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/klipper-rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

This project is a `no_std` MCU firmware for 3D printers running Klipper, implemented in pure Rust. It aims to provide a safe, modern, and high-performance alternative to the traditional C-based Klipper MCU firmware. The project is structured as a Rust workspace, containing multiple crates that work together to provide a complete firmware solution.

## Features

- **STM32F407xx Support**: Initial target is the MKS SKIPR board.
- **Async/Await with Embassy**: Uses the `embassy` framework for cooperative multitasking.
- **RTIC Support (Optional)**: Can be compiled with the `rtic` real-time framework as an alternative scheduler.
- **Core Klipper Functionality**:
  - Protocol parsing and command dispatch.
  - Low-latency stepper motor control queue.
  - ADC sampling for thermistors.
  - PWM heater control with PID loop.
- **Safety First**:
  - Watchdog timers to prevent lockups.
  - Thermal runaway protection and sensor failure detection.
  - Written in Rust for memory safety.

## Workspace Structure

This repository is a Cargo workspace that contains the following key crates:

- `klipper-mcu-firmware`: The main firmware application that runs on the MCU.
- `klipper-host`: Host-side utilities for communication and debugging (if applicable).
- `klipper-proto`: Defines the Klipper protocol data structures.
- `motion`: Motion control logic, including stepper planning.
- `mcu-drivers`: Hardware Abstraction Layer (HAL) and drivers for MCUs.
- `compat-layer`: Compatibility layer for C-based Klipper components.
- `sim`: A simulator for running and testing the firmware on a host machine.

For more details, see the `README.md` file in each crate's directory.

## Getting Started

### Prerequisites

- **Rust Toolchain**: Install Rust via `rustup`.
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Target Support**: Add the ARM Cortex-M target.
  ```bash
  rustup target add thumbv7em-none-eabihf
  ```
- **Flashing Tools**:
  - `probe-rs`: For flashing and debugging.
    ```bash
    cargo install probe-rs --features cli
    ```
  - `arm-none-eabi-objcopy`: For creating SD card binaries (part of `arm-none-eabi-gcc`).

### Building and Flashing

1. **Clone the Repository**:
   ```bash
   git clone <repository-url>
   cd klipper-rust
   ```

2. **Build the Firmware**:
   ```bash
   # Build the firmware for a specific board (e.g., mks_skipr)
   cargo build --release -p klipper-mcu-firmware --features mks_skipr
   ```

3. **Flash the Firmware**:
   Connect a debug probe (like an ST-Link) to your MCU and run:
   ```bash
   cargo embed --release -p klipper-mcu-firmware --features mks_skipr
   ```

## Heater Safety Notice

**WARNING**: The heater control code includes safety interlocks, but testing it carries risks. Improper configuration or bugs could lead to overheating and a fire hazard.

### Safe Testing Procedure

1. **Test without Power**: Before connecting the heater power supply, flash the firmware and connect to Klipper. Use a heat gun to gently warm the thermistor and verify that temperature readings are correct.
2. **Initial Power-On**: Connect a current-limited power supply to the heater circuit. Set a low current limit (e.g., 1A).
3. **PID Calibration**: Run Klipper's `PID_CALIBRATE` command. Monitor the temperature and current draw closely.
4. **Verify Failsafes**:
   - Disconnect the thermistor: The firmware should immediately shut down the heater.
   - Short the thermistor: The firmware should also detect this as an error and disable the heater.

**Do not leave the printer unattended during these tests.**

## Documentation

- [Architecture Overview](./docs/architecture.md)
- [Developer Workflow](./docs/dev_workflow.md)
- [Contributor Guide](./docs/contributors.md)

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.