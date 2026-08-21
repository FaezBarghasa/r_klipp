# r_klipp Developer & Architecture Guide

Welcome to the `r_klipp` developer guide. This guide explains the architecture, crate structure, build workflows, simulator usage, and target flashing procedures.

---

## 1. Workspace Crate Overview

```
r_klipp/
├── crates/
│   ├── parser/               # Zero-allocation G-Code lexer & parser (no_std)
│   ├── motion/               # Trapezoidal & 7-segment S-Curve motion planner, lookahead queue
│   ├── kinematics/           # Cartesian, CoreXY, and Delta kinematics models
│   ├── thermal/              # PID controller, host-assisted MPC, thermal runaway protection
│   ├── safety/               # Hardware watchdog, error supervisor, and E-Stop logic
│   ├── comms/                # Lock-free SPSC message channels and protocol bridge
│   ├── klipper-proto/        # Versioned postcard encoding, DPLL clock synchronization
│   ├── hal/                  # embedded-hal traits & peripheral abstractions
│   ├── mcu-drivers/          # Stepper DMA engine, endstop interrupt handlers, TMC drivers
│   ├── sim/                  # Deterministic host simulator with MockHal & CSV/SVG visualizers
│   ├── compat-layer/         # Klipper printer.cfg parser, strict schema validation & state machine
│   └── klipper-mcu-firmware/ # Embassy/RTIC bare-metal firmware for STM32 / RP2040
├── host-server/              # Moonraker-compatible REST & WebSocket server (Actix-Web)
└── host-ui/                  # Slint-based local touch screen UI (for Raspberry Pi / KlipperScreen)
```

---

## 2. Quickstart: Building and Running the Simulator

The `sim` crate runs entirely on your host OS (Linux, macOS, Windows) without needing physical MCU hardware.

### Running the Simulator Harness
```bash
# Run simulator unit tests
cargo test -p sim -- --nocapture

# Run the G-Code ingestion and trajectory exporter pipeline
cargo test -p sim test_gcode_pipeline_with_export -- --nocapture
```

### Exporting and Inspecting Trajectories
The simulator can export toolhead path, velocity, acceleration, and jerk profiles to CSV and SVG:
- **CSV Output**: `target/sim_trajectory.csv`
- **SVG Output**: `target/sim_toolpath.svg`

---

## 3. Building & Flashing Embedded Firmware

### Target Architecture Setup
Install the ARM Cortex-M target toolchain:
```bash
rustup target add thumbv7em-none-eabihf
cargo install probe-rs-tools
```

### Building for STM32F4 (e.g., MKS SKIPR / STM32F407)
```bash
cargo build --release -p klipper-mcu-firmware --target thumbv7em-none-eabihf --features embassy-rt
```

### Flashing via Probe-RS (ST-Link / J-Link / CMSIS-DAP)
```bash
probe-rs run --chip STM32F407VETx target/thumbv7em-none-eabihf/release/klipper-mcu-firmware
```

### Flashing via DFU (USB Bootloader)
```bash
cargo objcopy -p klipper-mcu-firmware --release --target thumbv7em-none-eabihf -- -O binary target/firmware.bin
dfu-util -a 0 -s 0x08000000:leave -D target/firmware.bin
```

---

## 4. Running Quality Assurance Tests

### Running Property-Based Tests
```bash
cargo test --test proptest_motion -- --nocapture
cargo test --test proptest_parser -- --nocapture
```

### Running Fuzzing (Parser)
```bash
# Optional: with cargo-fuzz
cargo install cargo-fuzz
cargo fuzz run fuzz_gcode_parser -- -max_total_time=60
```
