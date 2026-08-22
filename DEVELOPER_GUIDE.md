# Developer & Contributor Architecture Guide

Welcome to the **`r_klipp`** Developer Guide. This document provides technical insights, workspace architecture details, embedded flashing instructions, simulator workflows, and code hygiene practices.

---

## 🏗️ 1. Workspace Topology & Architecture

```
r_klipp/
├── crates/
│   ├── hal/                  # Universal embedded-hal traits (StepperAxis, DigitalOutput, etc.)
│   ├── parser/               # Zero-allocation streaming G-Code lexer and parser (no_std)
│   ├── motion/               # G4 31-phase trajectory generator, PH Bezier corners, CSS, Sync I/O
│   ├── kinematics/           # Cartesian, CoreXY, Delta, Dual-Head PnP, 5-Axis RTCP kinematics
│   ├── thermal/              # Kalman-filtered MPC & PID thermal controllers, runaway guards
│   ├── safety/               # Hardware IWDG, G38.2 probe supervisors, door interlocks
│   ├── comms/                # Lock-free SPSC channels and protocol bridges
│   ├── klipper-proto/        # Postcard serialization, COBS framing, DPLL clock synchronizer
│   ├── mcu-drivers/          # Embedded drivers (TMC2209/TMC2240, thermistors, endstops)
│   ├── sim/                  # Deterministic hardware simulator & SVG/CSV trajectory visualizer
│   ├── compat-layer/         # Multi-target configuration profiles and migration engines
│   └── klipper-mcu-firmware/ # Embedded bare-metal Embassy/RTIC firmware for STM32 & RP2040
├── host-server/              # Actix-Web + SurrealDB Moonraker and OpenPnP REST/WS server
├── host-ui/                  # Slint 1.x touchscreen operator interface
└── profiles/                 # Target machine configurations (3d_printer, pnp, cnc)
```

---

## 🚀 2. Developer Workflows & Quality Verification

### 2.1 Cargo Aliases
Preconfigured aliases in `.cargo/config.toml`:

```bash
# Run strict Clippy linter across host and core crates
cargo lint

# Run Clippy targeting embedded Cortex-M targets
cargo lint-mcu

# Check code formatting across the entire workspace
cargo fmt-check

# Run security and license audit
cargo deny-check

# Run test suite across all core logic crates
cargo test-core
```

### 2.2 Property-Based & Determinism Tests
`r_klipp` enforces mathematical determinism on all trajectory planners and parsers using `proptest`:

```bash
# Run motion planner property tests (acceleration bounds, finite jerk)
cargo test --test proptest_motion -- --nocapture

# Run G-Code parser fuzzer and invariant tests
cargo test --test proptest_parser -- --nocapture

# Verify cross-platform bit-identical trajectory determinism
cargo test --test determinism_test -- --nocapture
```

---

## 💻 3. Trajectory Simulator & Telemetry Analysis

The `sim` crate enables full-pipeline offline testing without physical MCU hardware:

```bash
# Run simulation pipeline and export toolpath visualizations
cargo test -p sim test_gcode_pipeline_with_export -- --nocapture
```

### Generated Artifacts:
- **`target/sim_trajectory.csv`**: Contains per-step timestamp, position $(X, Y, Z, E)$, instantaneous velocity $v(t)$, acceleration $a(t)$, and jerk $j(t)$.
- **`target/sim_toolpath.svg`**: Rendered vector graphic of the calculated toolpath with cornering velocity color mappings.

---

## ⚡ 4. Embedded Firmware Compilation & Flashing

### 4.1 Target Setup
Install the ARM Cortex-M cross-compilation toolchain:
```bash
rustup target add thumbv7em-none-eabihf
cargo install probe-rs-tools
```

### 4.2 Building for STM32F4 (e.g., MKS SKIPR / STM32F407)
```bash
cargo build --release -p klipper-mcu-firmware --target thumbv7em-none-eabihf --features embassy-rt
```

### 4.3 Flashing via Probe-RS (ST-Link / J-Link / CMSIS-DAP)
```bash
probe-rs run --chip STM32F407VETx target/thumbv7em-none-eabihf/release/klipper-mcu-firmware
```

### 4.4 Flashing via DFU (USB Bootloader)
```bash
cargo objcopy -p klipper-mcu-firmware --release --target thumbv7em-none-eabihf -- -O binary target/firmware.bin
dfu-util -a 0 -s 0x08000000:leave -D target/firmware.bin
```

---

## 🌐 5. Host Server & UI Development

### Running the Server & UI Locally
```bash
# Launch the Actix-Web + SurrealDB server and Slint Touch UI
cargo run -p host-server
```

- **Moonraker API**: Accessible at `http://127.0.0.1:7125/printer/info`
- **WebSocket Telemetry**: Stream at `ws://127.0.0.1:7125/websocket`
- **OpenPnP Bridge**: Supports G-Code socket commands on standard configured ports.
