# Klipper in Rust: Simulator Crate

## Overview

The `sim` crate provides a simulator for the Klipper in Rust MCU firmware. It allows the firmware logic to be compiled and run as a native application on a host machine (e.g., a PC).

This is an invaluable tool for development, debugging, and testing, as it provides a much faster and more convenient development cycle than testing on physical hardware.

## Features

*   **Hardware Abstraction**: Provides a simulated `Hardware Abstraction Layer (HAL)` that mimics the behavior of the real MCU peripherals. For example, it provides a simulated UART that can be connected to a pseudo-terminal, and simulated GPIOs that can be toggled and monitored.
*   **Host Integration**: The simulator can be connected to host-side test scripts or a simulated Klipper host, allowing for end-to-end testing of the entire software stack.
*   **Logging and Debugging**: When running in the simulator, logs can be printed directly to the console, and standard debugging tools like `gdb` or `lldb` can be used.
*   **No Hardware Required**: Allows for the development and testing of large parts of the firmware without needing access to a physical 3D printer or MCU board.

## Usage

To run the simulator, simply execute `cargo run` from within this crate's directory.

```bash
cd crates/sim
cargo run
```

The simulator will start and listen for a connection from a host application.

## How it Works

The simulator works by using conditional compilation (`#[cfg(...)]`) to replace the real hardware drivers with simulated versions when the `sim` feature is enabled.

The main application logic in `klipper-mcu-firmware` is written against the generic `embedded-hal` traits, so it can be compiled with either the real HAL for the target MCU or the simulated HAL provided by this crate. This allows the same high-level application code to run in both environments.
