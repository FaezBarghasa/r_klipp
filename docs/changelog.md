# Changelog

All notable changes to the `r_klipp` project will be documented in this file. This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.5.1] - 2026-06-02

This is a maintenance release that modernizes dependencies, improves the CI pipeline, and refactors several internal APIs for clarity and safety.

### Changed
- **Dependency Modernization**: Updated all Rust dependencies to their latest stable versions, including a move to the latest `nom` parser and `rand` crate APIs.
- **CI Pipeline Improvements**: The CI pipeline now runs Clippy with target-specific checks for embedded firmware, ensuring higher code quality for both host and MCU targets.
- **API Refinements**:
    - The `StepperController::execute_next_step_isr` function now includes explicit safety documentation for its use of raw pointers.
    - The `DmaStepEngine` now implements `Default`.
    - Minor cleanup in the `thermistor` module.

---

## [0.5.0] - 2026-06-02

This release introduces major architectural overhauls, migrating core utilities to standard community-backed crates, optimizing kinematics with advanced curve profiling, implementing a State-Space thermal regulator, and establishing a robust testing framework.

### Added
- **Standard Fixed-Point Representation**: Integrated the `fixed` crate. Replaced custom fixed-point code with the standard `fixed::types::I16F16` wrapped in `Fixed16_16` using saturating arithmetic.
- **Degree-15 Pythagorean-Hodograph (PH) Blending**: Introduced a $C^4$ continuous spline corner blender (`PhBezier15` inside `crates/motion`) featuring Kahan compensated summation and Newton-Raphson parameter solvers.
- **G^4 31-Phase Trajectory Generator**: Added high-order kinematic planning (`G4Profile` inside `crates/motion`) to restrict crackle, snap, jerk, and acceleration across 31 motion phases.
- **State-Space MPC Thermal Regulation**: Introduced a Kalman Filter state estimator and Model Predictive Control thermal controller (`MpcThermalEngine` in `crates/thermal`) with volumetric flow rate feed-forward loss compensation.
- **Self-Describing Pinout Configuration Protocol**: Implemented an automated serial handshake manifest (`HandshakeManifest` in `crates/klipper-proto`) supporting zero-copy postcard parsing.
- **Sandboxed Rhai Scripting VM**: Added host-side G-Code macro sandboxing (`HostMacroEngine` in `crates/klipper-host`) with execution instruction limits (100,000 max operations).
- **Double-Buffered DMA Step Offloading**: Added `DmaStepEngine` in `crates/mcu-drivers` for high-frequency (1 MHz) stepper pulse scheduling.
- **Lock-Free Multi-MCU Clock Sync**: Introduced recursive least-squares regression mapping (`ClockSyncModel` in `crates/klipper-mcu-firmware`) with lock-free atomic buffer swaps (`SharedClockModel`).
- **HIL Vibration Analysis**: Added `InputShaperCalibrator` inside `crates/klipper-host` to compute Discrete Fourier Transforms (DFT) of resonance peaks and solve ZV and MZV vibration filters.
- **Unit and Integration Testing Framework**: Introduced comprehensive test suites and diagnostic helper methods across host, thermal, motion, and driver modules.

### Changed
- **Step Queue Renaming**: Renamed `StepCommand` structure to `StepSegment` and refactored its fields to align with the lock-free SPSC ring buffers.
- **UART Driver Upgrades**: Migrated MCU UART drivers to use `embedded-hal-nb` (non-blocking traits) for better hardware compatibility.
- **Mock Driver Refactoring**: Migrated `embedded-hal-mock` dependencies to the new `embedded-hal` v1.0.0 digital module configuration.
- **no-std Compatibility**: Relied on postcard serialization and heapless structures for zero-allocation compatibility.
- **Dependency Upgrades**: Cleaned and upgraded all Cargo dependencies to their latest stable releases across all workspace crates.

---

## [0.4.0] - 2026-05-15

This release adds support for high-performance ARM Cortex-M4 architectures, improves configuration parsing safety, and refactors message-passing lifetime bounds.

### Added
- **STM32F407xx Architecture Support**: Integrated target features and pinout configurations for the MKS SKIPR mainboard.
- **Lifetime-Bound Command Processing**: Refactored `parse_command` to accept lifetime-bound `Command` structures, avoiding unnecessary copies.
- **Enhanced Configuration Error Handling**: Introduced detailed parser errors when reading G-Code configuration files.

### Changed
- **Protocol Parsing Safety**: Overhauled the `klipper-proto` deserialization loops to guard against buffer overruns and corrupted CRC frames.

---

## [0.3.0] - 2026-04-01

Initial prototype release containing embassy integration, low-level stepper timers, and basic ADC thermistor reading.

### Added
- **Embassy Asynchronous Executor**: Configured cooperative async multitasking for the MCU main loop.
- **Basic PID Controller**: Implemented standard PID heater loops for the hotend and heated bed.
- **GPIO & PWM Peripheral Drivers**: Wrote basic peripheral controls for stepper enable, step, direction, and fan PWM lines.
