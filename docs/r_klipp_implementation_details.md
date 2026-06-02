# r_klipp Modules Implementation Details

This document provides the technical specification and architectural blueprint for the implementation of the advanced control systems, real-time safety engines, and bare-metal abstractions within the `r_klipp` workspace.

## Phase 1 Specifications
- **Dynamic SPSC Lock-Free Step Queue**: Implemented in `mcu-drivers/src/stepper.rs` to schedule steps without lock contention overhead.
- **Hardware Timer + DMA Step Offloading**: Handled by `DmaStepEngine` in `mcu-drivers/src/stepper_dma.rs` to alleviate interrupt overhead.
- **Self-Describing Pinout Configuration**: Established `HandshakeManifest` in `klipper-proto/src/autoconfig.rs` for dynamic host-side auto-assignment without static driver definitions.
- **Rhai Scripting VM Integration**: In `klipper-host/src/macro_engine.rs`, encapsulating high-level script execution from stepping loops.

## Phase 2 Specifications
- **Degree-15 Pythagorean-Hodograph Corner Blending**: Implemented analytical spline evaluation in `motion/src/ph_beziers.rs`.
- **G4 Motion Profile Planner**: Bound kinematic trajectory transitions across 31 continuous phases within `motion/src/g4_planner.rs`.
- **Overshoot-Constrained Homing**: Implemented scaling crash deceleration algorithms in `motion/src/homing.rs`.
- **State-Space MPC Thermal Control**: The prediction matrices and Kalman corrections reside in `thermal/src/mpc.rs`.

## Phase 3 Specifications
- **Multi-MCU Clock Synchronization**: The DPLL timeline translation logic lives within `klipper-mcu-firmware/src/clock_sync.rs`.
- **Dynamic HIL Benchmarking & Noise Profiling**: Fourier coefficient extraction routines are in `klipper-host/src/hil_analyzer.rs`.
