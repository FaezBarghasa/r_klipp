# Klipper MCU Firmware in Rust

[![CI](https://github.com/your-username/klipper-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/klipper-rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

This project is a `no_std` MCU firmware for 3D printers running Klipper, implemented in pure Rust. It provides a safe, modern, and high-performance alternative to the traditional C-based Klipper MCU firmware. The project is structured as a Rust workspace, containing multiple crates that work together to provide a complete firmware solution.

Our philosophy is to leverage Rust's safety and concurrency features to create a reliable and maintainable firmware. We use modern asynchronous programming with `embassy` to handle complex, timing-sensitive tasks without the need for a traditional real-time operating system.

## Features

- **Multi-Platform Support**: Designed to be portable across different MCU architectures, with initial support for STM32F407xx (e.g., MKS SKIPR board).
- **Asynchronous Execution**: Built on the `embassy` async framework for cooperative multitasking, enabling efficient, non-blocking I/O.
- **Real-Time Control**: Optional support for the `rtic` framework for developers who prefer a hardware-interrupt-driven, real-time scheduling model.
- **Core Klipper Functionality**:
  - **Protocol Engine**: A robust parser for the Klipper protocol, with efficient command dispatch.
  - **Motion Control**: A precise, low-latency stepper motor control queue for high-speed printing.
  - **Sensor Integration**: ADC sampling for thermistors with support for various sensor types.
  - **Heater Management**: PWM heater control with a PID feedback loop for stable temperature regulation.
- **Uncompromising Safety**:
  - **Watchdog Timers**: System-wide hardware and software watchdogs to prevent lockups and ensure continuous operation.
  - **Thermal Protection**: Advanced thermal runaway protection and sensor failure detection to prevent fire hazards.
  - **Memory Safety**: Written in Rust to eliminate entire classes of bugs like buffer overflows and data races.

## Workspace Structure

This repository is a Cargo workspace that contains the following key crates:

- `klipper-mcu-firmware`: The main firmware application that runs on the MCU.
- `klipper-host`: Host-side utilities for communication and debugging.
- `klipper-proto`: Defines the Klipper protocol data structures and serialization/deserialization logic.
- `motion`: Motion control algorithms, including stepper pulse generation and kinematics.
- `mcu-drivers`: Hardware Abstraction Layer (HAL) and drivers for various MCUs and peripherals.
- `compat-layer`: A compatibility layer to interface with existing C-based Klipper components or libraries.
- `sim`: A simulator for running and testing the firmware on a host machine, enabling rapid development and debugging.

For more details, see the `README.md` file in each crate's directory.

## Getting Started

### Prerequisites

- **Rust Toolchain**: Install Rust using `rustup`.
  ```bash
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
  ```
- **Target Support**: Add the appropriate ARM Cortex-M target for your board. For example, for an STM32F4:
  ```bash
  rustup target add thumbv7em-none-eabihf
  ```
- **Flashing and Debugging Tools**:
  - `probe-rs`: A powerful tool for flashing and debugging embedded Rust applications.
    ```bash
    cargo install probe-rs --features cli
    ```
  - `arm-none-eabi-objcopy`: Required for creating binaries for SD card flashing (part of the ARM GCC toolchain).

### Building and Flashing

1. **Clone the Repository**:
   ```bash
   git clone <repository-url>
   cd klipper-rust
   ```

2. **Build the Firmware**:
   Build the firmware for a specific board by enabling its feature flag. For example, for the `mks_skipr` board:
   ```bash
   cargo build --release -p klipper-mcu-firmware --features mks_skipr
   ```

3. **Flash the Firmware**:
   Connect a debug probe (e.g., an ST-Link or J-Link) to your MCU and use `cargo embed`:
   ```bash
   cargo embed --release -p klipper-mcu-firmware --features mks_skipr
   ```

## Heater and Motion System Safety Notice

**WARNING**: The heater and motion control code includes critical safety interlocks, but testing it on a live machine carries significant risks. Improper configuration, bugs, or hardware failures could lead to overheating, fire hazards, or unexpected machine movement.

### Safe Testing Procedure

1. **Test without Power**: Before connecting the heater or motor power supply, flash the firmware and connect to Klipper. Use a heat gun to gently warm the thermistor and verify that temperature readings are correct. Manually move the axes and check that endstops report the correct status.
2. **Initial Power-On**: Connect a current-limited power supply to the heater and motor circuits. Set a low current limit (e.g., 1A) to minimize the risk of damage.
3. **PID and Motor Calibration**: Run Klipper's `PID_CALIBRATE` command and motor tuning procedures. Monitor temperatures and currents closely.
4. **Verify Failsafes**:
   - **Thermistors**: Disconnect and short the thermistor connectors. The firmware must immediately shut down the corresponding heater.
   - **Endstops**: Trigger the endstops manually and verify that motion commands are halted.
   - **Emergency Stop**: Test the emergency stop functionality from your Klipper host.

**Do not leave the printer unattended during these tests or initial prints.**

## Project Documentation

- [Project Architecture](./docs/architecture.md)
- [Developer Workflow](./docs/dev_workflow.md)
- [Contributor Guide](./docs/contributors.md)
- [Hardware Setup Guide](./docs/hardware_setup.md)
- [Protocol Details](./docs/protocol.md)
- [Safety Subsystem](./docs/safety.md)

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.
