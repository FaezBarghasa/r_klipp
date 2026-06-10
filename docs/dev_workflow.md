# Developer Workflow

This document outlines the recommended workflow for developing, testing, and profiling the `r_klipp` firmware. Following these steps ensures code quality, consistency, and a smooth development process.

---

## 1. Setting Up Your Environment

### Prerequisites

*   **Rust Toolchain**: Make sure you have the latest stable version of Rust installed via `rustup`.
*   **Target Architecture**: Install the appropriate target for your MCU (e.g. `thumbv7em-none-eabihf` for STM32F4).
*   **Embedded Tools**:
    *   `cargo-embed`: For flashing and viewing RTT logs.
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

---

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
2.  **Check for Compile Errors**: Ensure the build completes without warnings or errors.

### Step 2.3: Flashing and Debugging

`cargo-embed` simplifies the process of flashing the firmware and viewing logs.

1.  **Connect the Debug Probe**: Connect the probe to your MCU's SWD port and your computer.
2.  **Start the Embed Session**:
    ```bash
    cargo embed --release -p klipper-mcu-firmware --features mks_skipr
    ```
    This command will compile the firmware, flash it to the MCU, and open an RTT (Real-Time Transfer) terminal to display logs.

### Step 2.4: Logging and Tracing

The firmware uses the `defmt` crate for efficient, low-overhead logging.
*   **Log Levels**: Use the `defmt` macros (`defmt::info!`, `defmt::warn!`, `defmt::error!`, etc.) to add logs to your code.
*   **Viewing Logs**: Logs will automatically appear in the `cargo-embed` terminal. You can control the log level using the `DEFMT_LOG` environment variable.

---

## 3. Testing

A robust testing strategy is essential for ensuring the firmware is reliable.

### Unit and Integration Tests

The workspace features standard unit tests across most modules (host, thermal, motion, and driver modules).

*   **Running Crate Tests**:
    ```bash
    # Run tests for a specific crate (e.g. motion)
    cargo test -p motion
    ```
*   **Running Workspace Tests**:
    Due to the `no_std` nature of the firmware, workspace tests should exclude the firmware and bare-metal driver crates:
    ```bash
    # Run tests in the workspace (excluding embedded components)
    cargo test --workspace --exclude klipper-mcu-firmware --bins --lib --tests
    ```

### Hardware-in-the-Loop (HIL) Testing

HIL testing is critical for validating the firmware's interaction with real-world hardware and measuring its real-time performance characteristics.
Detailed setup instructions, logic analyzer layouts, and end-to-end latency measurement workflows are located in the [HIL Testing Plan](./hil_testing.md) guide.

---

## 4. Performance Profiling

To identify CPU bottlenecks ("hot spots") in the host or firmware components, developers can profile execution via cargo-flamegraph. Detailed instructions are in [Performance Profiling](./profiling.md).

### 4.1. Profiling the Host (`klipper-host`)
```bash
cargo flamegraph --release -p klipper-host -- run crates/compat-layer/tests/ender3.cfg
```

### 4.2. Profiling the Simulator (`sim`)
Since `perf` cannot run directly on the MCU, profile the firmware logic under the host simulator:
```bash
cargo flamegraph --release -p sim --example run_sim
```

---

## 5. Code Quality

Before committing or submitting a pull request, run the following quality checks:

1.  **Formatting**:
    ```bash
    cargo fmt --all
    ```
2.  **Linting**:
    Ensure that you run target-specific linting to verify both host code and bare-metal firmware code.
    ```bash
    # Check Host & Simulator Crates
    cargo clippy --workspace --exclude klipper-mcu-firmware --exclude mcu-drivers -- -D warnings
    
    # Check Embedded Library & Firmware Target
    cargo clippy -p klipper-mcu-firmware -p mcu-drivers --target thumbv7em-none-eabihf --features embassy-rt -- -D warnings
    ```
