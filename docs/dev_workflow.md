# Developer Workflow

This document outlines the recommended workflow for developing and testing the Klipper in Rust firmware. Following these steps will help ensure code quality, consistency, and a smooth development process.

## 1. Setting Up Your Environment

### Prerequisites

*   **Rust Toolchain**: Make sure you have the latest stable version of Rust installed via `rustup`.
*   **Target Architecture**: Install the appropriate target for your MCU (e.g., `thumbv7em-none-eabihf`).
*   **Embedded Tools**:
    *   `cargo-embed`: For easy flashing and debugging.
    *   `probe-rs`: The underlying tool used by `cargo-embed`.
    *   `flip-link`: To enable stack overflow protection.
*   **Hardware**:
    *   A supported MCU board (e.g., MKS SKIPR).
    *   A debug probe (e.g., ST-Link v2/v3 or J-Link).

### Installation

```bash
# Install essential tools
cargo install cargo-embed probe-rs flip-link

# Add the target for your MCU
rustup target add thumbv7em-none-eabihf
```

## 2. The Development Cycle

### Step 2.1: Running in the Simulator

The simulator allows you to test much of the firmware's logic on your host machine, which is much faster than flashing to a physical board.

1.  **Navigate to the Simulator Crate**:
    ```bash
    cd crates/sim
    ```

2.  **Run the Simulator**:
    ```bash
    cargo run --release
    ```

The simulator will start and can be connected to a host application or test script to validate functionality like protocol parsing and command handling.

### Step 2.2: Building for the Target

When you're ready to test on hardware, you need to build the firmware for your specific board.

1.  **Select the Board Feature**: The firmware uses Cargo features to configure the build for a specific board. For example, to build for the `mks_skipr`:
    ```bash
    cargo build --release -p klipper-mcu-firmware --features mks_skipr
    ```
2.  **Check for Compile Errors**: Ensure the build completes without any errors or warnings.

### Step 2.3: Flashing and Debugging

`cargo-embed` simplifies the process of flashing the firmware and viewing logs.

1.  **Connect the Debug Probe**: Connect the probe to your MCU's SWD port and your computer.
2.  **Start the Embed Session**:
    ```bash
    cargo embed --release -p klipper-mcu-firmware --features mks_skipr
    ```
    This command will:
    *   Compile the firmware.
    *   Flash it to the MCU.
    *   Open an RTT (Real-Time Transfer) terminal to display logs.

### Step 2.4: Logging and Tracing

The firmware uses the `defmt` crate for efficient, low-overhead logging.

*   **Log Levels**: Use the `defmt` macros (`defmt::info!`, `defmt::warn!`, `defmt::error!`, etc.) to add logs to your code.
*   **Viewing Logs**: Logs will automatically appear in the `cargo-embed` terminal. You can control the log level using the `DEFMT_LOG` environment variable.

## 3. Testing

A robust testing strategy is essential for ensuring the firmware is reliable.

### Unit and Integration Tests

*   **Location**: Tests are located in the `tests/` directory of each crate.
*   **Running Tests**:
    ```bash
    # Run tests for a specific crate
    cargo test -p klipper-mcu-firmware

    # Run all tests in the workspace
    cargo test --workspace
    ```

### Hardware-in-the-Loop (HIL) Testing

For functionality that depends on real hardware, we use HIL tests.

*   **Framework**: We use `probe-rs-test` to run tests directly on the target MCU.
*   **Running HIL Tests**:
    ```bash
    # (Instructions for running HIL tests will be added here)
    ```

## 4. Code Quality

Before submitting your changes, please run the following checks:

1.  **Formatting**:
    ```bash
    cargo fmt --all
    ```
2.  **Linting**:
    ```bash
    cargo clippy --workspace -- -D warnings
    ```

## 5. Submitting a Pull Request

Once your changes are complete and have passed all checks, you can open a pull request. Please refer to the [Contributor Guide](./contributors.md) for detailed instructions on the PR process.
