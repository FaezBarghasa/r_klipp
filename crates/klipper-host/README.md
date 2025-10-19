# Klipper in Rust: Host Utilities Crate

## Overview

The `klipper-host` crate provides a collection of host-side utilities for interacting with, testing, and debugging the Klipper in Rust MCU firmware.

This crate is intended to be used on a development machine (e.g., a PC running Linux, macOS, or Windows) and is not part of the MCU firmware itself.

## Features

*   **Host-Side Protocol Implementation**: Contains a Rust implementation of the Klipper protocol, allowing for the creation of host applications that can communicate with the MCU.
*   **Test Utilities**: Provides tools for writing integration and end-to-end tests for the firmware. This includes utilities for sending specific command sequences and asserting the correctness of the MCU's responses.
*   **Debugging Tools**: Includes command-line tools for monitoring the MCU's state, inspecting logs, and sending raw protocol messages for debugging purposes.
*   **Firmware Flashing**: May include functionality to flash the firmware to the target MCU, acting as a wrapper around tools like `probe-rs`.

## Usage

This crate can be used as a library for building custom host applications or as a collection of command-line tools.

### As a Library

```rust
// Example of using the host library to connect to the MCU
use klipper_host::protocol::KlipperConnection;

async fn connect_and_run_tests() {
    let mut conn = KlipperConnection::new("/dev/ttyACM0").await.unwrap();
    conn.send_command("GET_VERSION").await.unwrap();
    let response = conn.read_response().await.unwrap();
    println!("MCU Version: {}", response);
}
```

### Command-Line Tools

```bash
# Example of using a CLI tool from this crate
cargo run -p klipper-host -- tool get-temp -h /dev/ttyACM0
```
