Detailed Implementation Specification: r_klipp Modules

This document provides an exhaustive, low-level technical specification and architectural blueprint for the implementation of the advanced control systems, real-time safety engines, and bare-metal abstractions within the r_klipp workspace.

1. System Directory & Workspace Crate Architecture

To ensure separation of concerns, compile-time memory optimization, and strict safety boundaries between the high-level planning host (klipper-host) and the bare-metal microcontroller target (klipper-mcu-firmware), the workspace is organized as follows:

r_klipp/  (Master Workspace Root)
├── Cargo.toml                  <-- Master workspace manifest
├── docs/
│   └── r_klipp_implementation_details.md  <-- This document
├── crates/
│   ├── motion/                 <-- Shared motion profiling and kinematics
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── ph_beziers.rs   <-- Degree-15 Pythagorean-Hodograph blender
│   │       └── g4_planner.rs   <-- 31-Phase G4 trajectory solver
│   ├── thermal/                <-- Shared thermal modeling and state estimation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── mpc.rs          <-- State-space MPC and Kalman filter
│   ├── klipper-proto/          <-- Serialization, transport packets, autoconfig
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       └── autoconfig.rs   <-- Handshake payloads and manifests
│   ├── mcu-drivers/            <-- Low-level peripheral controllers (no_std)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── stepper.rs      <-- Step timers, registers, DMA controller
│   │       └── probe.rs        <-- Safe probe state machines
│   └── safety/                 <-- Strict compile-time safety guards
│       ├── Cargo.toml
│       └── src/
│           └── lib.rs
├── crates/klipper-host/         <-- High-level orchestrator (std)
│   ├── Cargo.toml
│   └── src/
│       ├── main.rs
│       ├── macro_engine.rs     <-- Sandboxed Rhai script engine
│       └── config.rs           <-- Dynamic deserializers
└── crates/klipper-mcu-firmware/ <-- Target firmware (no_std, RTIC 2 + Embassy)
    ├── Cargo.toml
    └── src/
        ├── main.rs
        └── clock_sync.rs       <-- RTIC 2 Priority 4 PLL Clock sync


Part 2: Phase 1 (Short Term) Specifications

Task 1.1: Dynamic SPSC Lock-Free Step Queue
Target Crate: mcu-drivers
Target File: crates/mcu-drivers/src/stepper.rs

PM Requirements & Acceptance Criteria:
Maximum Jitter: Must ensure step scheduling overhead does not exceed 20 CPU clock cycles (83 ns at 240 MHz).
Lock-Free Safety: Under no circumstances may the stepping interrupt lock, block, or spin. Memory transfer must execute within a single clock cycle using atomic pointer swaps.
Memory Footprint: Fixed allocation of 2048 step segments per motor to prevent heap fragmentation.
Design Pattern: Single-Producer Single-Consumer (SPSC) Lock-Free Ring Buffer. Communication layers push steps; hardware interrupts pull and update timer parameters.

Task 1.2: Self-Describing Pinout Configuration Protocol
Target Crate: klipper-proto
Target File: crates/klipper-proto/src/autoconfig.rs

PM Requirements & Acceptance Criteria:
Handshake Reliability: Must negotiate within 1.5 seconds of serial port enumeration.
Memory Efficiency: Zero-copy serialization to fit within small transport buffers (< 512 bytes).
Design Pattern: Self-Describing Manifest Pattern. The target MCU broadcasts its configuration capabilities, enabling the host to auto-assign resources without static driver files.

Task 1.3: Rhai Scripting VM Integration
Target Crate: klipper-host
Target File: crates/klipper-host/src/macro_engine.rs

PM Requirements & Acceptance Criteria:
Pre-Compilation: Macros must compile once during configuration parsing.
Safety Limits: Maximum instruction count per macro execution is capped at 100,000 operations.
Latency: Execution overhead must be less than 5µs per instruction.
Design Pattern: Sandboxed Virtual Machine Pattern. Encapsulates high-level scripting loops away from critical stepping threads.


Part 3: Phase 2 (Medium Term) Specifications

Task 2.1: Degree-15 Pythagorean-Hodograph (PH) Corner Blending Solver
Target Crate: motion
Target File: crates/motion/src/ph_beziers.rs

PM Requirements & Acceptance Criteria:
Tangent Continuity (C^4): Transitions between adjacent linear paths must be C^4 continuous.
Newton-Raphson Bounds: Root convergence of the speed polynomial must resolve down to < 10^-9 mm within 5 iterations.
Design Pattern: Analytical Spline Engine Pattern. Computes closed-form geometrical equations in real-time, removing numerical integration from execution loops.

Task 2.2: G^4 Motion Profile (31-Phase Trajectory Generator)
Target Crate: motion
Target File: crates/motion/src/g4_planner.rs

PM Requirements & Acceptance Criteria:
Continuous Acceleration & Jerk: The generated segments must bound both snap and crackle, preventing high-frequency mechanical shock.
Segment Output Limit: Calculated phases are capped at an array of 32 elements to fit within memory-mapped memory frames.
Design Pattern: Kinematic Pipeline Pattern. Progressively applies jerk, snap, and crackle restrictions to a discrete queue of movement vectors.

Task 2.3: State-Space MPC Thermal Control Module
Target Crate: thermal
Target File: crates/thermal/src/mpc.rs

PM Requirements & Acceptance Criteria:
Extrusion Feed-Forward: System must scale heating current ahead of physical plastic feed rate increases.
Estimation Robustness: The unmeasured heater core temperature T_heater estimate must remain stable and track sensors to within 0.5°C in steady state.
Design Pattern: State-Space Observer Pattern. Utilizes discrete mathematical matrices to predict system changes before they register on thermistors.

Task 2.4: Overshoot-Constrained Homing Math Module
Target Crate: motion
Target File: crates/motion/src/homing.rs

PM Requirements & Acceptance Criteria:
Crash Prevention: Homing speed must automatically scale down if the remaining physical coordinate distance is less than the calculated braking deceleration distance.
Frame Tolerance: Braking deceleration margin is locked to a safety factor of 0.85.
Design Pattern: Braking Curve Guard Pattern. Slices movement profiles to guarantee collision boundaries are never crossed.


Part 4: Phase 3 (Long Term) Specifications

Task 3.1: Multi-MCU Clock Synchronization (RTIC 2 Clock Sync)
Target Crate: klipper-mcu-firmware
Target File: crates/klipper-mcu-firmware/src/clock_sync.rs

PM Requirements & Acceptance Criteria:
Synchronization Target: Secondary MCU clocks must align with the primary host controller to within ±1µs over a continuous 24-hour print run.
NVIC Interrupt Preemption: Clock capture interrupts must run at Priority 4, bypassing background tasks.
Design Pattern: Distributed Phase-Locked Loop (DPLL) Strategy. Compares master timeframes with local timer capture registers.

Task 3.2: Hardware Timer + DMA Step Offloading
Target Crate: mcu-drivers
Target File: crates/mcu-drivers/src/stepper.rs (Note: Implemented as stepper_dma.rs)

PM Requirements & Acceptance Criteria:
Timer Frequency: Must sustain a 1 MHz step generation frequency without CPU starvation.
Alternate Function Mapping: Pins must toggle natively via hardware compare logic, completely eliminating ISR step execution lag.
Design Pattern: Hardware-Assisted Active Pattern. Decouples physical pin-state changes from real-time CPU task preemption.

Task 3.3: Dynamic HIL Benchmarking & Noise Profiling
Target Crate: klipper-host
Target File: crates/klipper-host/src/hil_analyzer.rs

PM Requirements & Acceptance Criteria:
Accurate Calibration: Dynamic Fourier extraction sweeps must cover a spectrum of 10 Hz to 150 Hz.
SPI Sampling Frequency: The accelerometer hardware interface must achieve a stable, deterministic SPI capture loop of 3.2 kHz.
Design Pattern: Observer / Spectrum Analyzer Pattern. Correlates high-speed accelerometer readings into dynamic motion filter adjustments.
