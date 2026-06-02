# Klipper MCU Firmware in Rust

[![CI](https://github.com/your-username/klipper-rust/actions/workflows/ci.yml/badge.svg)](https://github.com/your-username/klipper-rust/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

## Overview

This project is a `no_std` MCU firmware for 3D printers running Klipper, implemented in pure Rust. It provides a safe, modern, and high-performance alternative to the traditional C-based Klipper MCU firmware. The project is structured as a Rust workspace, containing multiple crates that work together to provide a complete firmware solution.

Our philosophy is to leverage Rust's safety and concurrency features to create a reliable and maintainable firmware. We use modern asynchronous programming with `embassy` to handle complex, timing-sensitive tasks, combined with `rtic` for interrupt-driven real-time scheduling.

---

## Features

- **Multi-Platform Support**: Portable across different MCU architectures, with initial support for STM32F407xx (e.g., MKS SKIPR board).
- **Dual-Paradigm Execution**: Cooperative multitasking with the `embassy` async executor alongside preemptive real-time interrupts via `rtic`.
- **Advanced Motion Control**:
  - **PH Corner Blending**: Continuous $C^4$ tangent transitions between linear moves using a degree-15 Pythagorean-Hodograph (`PhBezier15`) blender.
  - **G^4 31-Phase Trajectory Generator**: Smooth kinematic profiles bounding crackle, snap, jerk, and acceleration.
  - **DMA Double-Buffered Offloading**: Sustainable 1 MHz step generation using `DmaStepEngine` with zero CPU jitter.
  - **Braking Curve Guard Homing**: Safe homing scaling velocity based on deceleration capabilities and endstop proximity.
- **State-Space Thermal Management**:
  - **Kalman Filter State Estimation**: Estimating internal heater core temperatures and sensor readings using discrete state matrices.
  - **Feed-Forward Loop**: Volumetric extrusion rate feed-forward heater current scaling, eliminating temp drops during high-speed printing.
- **Self-Describing autoconfig Protocol**:
  - Automatically negotiates MCU capabilities (`HandshakeManifest`, `PinDescriptor`, `PinCapability`) with zero-copy postcard parsing.
- **Sandboxed Macro Engine**:
  - Host-side Rhai VM compiling macros to AST with instruction caps to prevent scheduler lockups.
- **Multi-MCU Clock Synchronization**:
  - Real-time time drift fitting using recursive least-squares regression with atomic lock-free shared models.
- **HIL Calibration & Input Shaping**:
  - Discrete Fourier Transform (DFT) resonance analysis on SPI accelerometer inputs to configure Zero-Vibration (ZV) and Minimum Zero-Vibration (MZV) filters.

---

## Workspace Structure

This repository is a Cargo workspace that contains the following key crates:

- `klipper-mcu-firmware`: The main `no_std` firmware application running on the MCU, utilizing RTIC 2 and Embassy.
- `klipper-host`: Host-side orchestrator managing G-Code interpretation, the Rhai macro engine, and HIL calibrators.
- `klipper-proto`: Serialization schemas (`postcard`) and capabilities handshake parameters.
- `motion`: Advanced kinematics solvers, PH Bezier blender, and the G4 profile generator.
- `thermal`: Kalman estimator and MPC thermal regulation loop.
- `mcu-drivers`: Low-level peripheral drivers, including SPSC step controllers and DMA buffers.
- `compat-layer`: Compatibility tools to interface with existing Klippy components.
- `sim`: MCU simulator for desktop debugging and fast developer testing.

---

## File Structure

```
.
├── boards/             # Board-specific configurations and pinouts
├── ci/                 # Continuous integration scripts and workflows
├── config/             # Example Klipper configuration files
├── crates/             # All Rust source code, organized as a Cargo workspace
│   ├── compat-layer/
│   ├── firmware-builder/
│   ├── klipper-host/
│   ├── klipper-mcu-firmware/ # The main firmware application
│   ├── klipper-proto/
│   ├── mcu-drivers/
│   ├── motion/
│   ├── sim/
│   └── thermal/
├── docs/               # In-depth documentation
├── tests/              # Integration and hardware-in-the-loop (HIL) tests
├── tools/              # Helper scripts and tools for development
├── .gitignore
├── Cargo.toml          # Workspace manifest
└── README.md
```

---

## Getting Started

### Prerequisites

- **Rust Toolchain**: Install Rust using `rustup`.
- **Target Support**: Add the appropriate ARM Cortex-M target (e.g. `thumbv7em-none-eabihf` for STM32F4).
- **Flashing Tools**: `probe-rs` or `cargo-embed`.

### Building and Flashing

1. **Build the Firmware**:
   ```bash
   cargo build --release -p klipper-mcu-firmware --features mks_skipr
   ```
2. **Flash via Debug Probe**:
   ```bash
   cargo embed --release -p klipper-mcu-firmware --features mks_skipr
   ```

---

## Heater and Motion System Safety Notice

**WARNING**: The heater and motion control code includes critical safety interlocks, but testing it on a live machine carries significant risks. Improper configuration, bugs, or hardware failures could lead to overheating, fire hazards, or unexpected machine movement.

Always read and perform the [First-Time Safe Testing Procedure](./docs/SAFETY_NOTICE.md) before connecting mains power.

---

## Project Documentation

- [Project Architecture](./docs/architecture.md)
- [System Architecture Diagrams](./docs/diagram.md)
- [Developer Workflow](./docs/dev_workflow.md)
- [Contributor Guide](./docs/contributors.md)
- [Hardware Setup Guide](./docs/hardware_setup.md)
- [Protocol Details](./docs/protocol.md)
- [Safety Subsystem](./docs/safety.md)
- [HIL Testing Plan](./docs/hil_testing.md)
- [Performance Profiling](./docs/profiling.md)
- [Changelog](./docs/changelog.md)

## License

This project is licensed under the MIT License. See the [LICENSE](./LICENSE) file for details.
