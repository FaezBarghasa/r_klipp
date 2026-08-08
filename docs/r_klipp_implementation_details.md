# Detailed Implementation Specification: r_klipp Modules

This document provides an exhaustive, low-level technical specification and architectural blueprint for the implementation of the advanced control systems, real-time safety engines, and bare-metal abstractions within the r_klipp workspace.

**Note:** This document is a snapshot of the project's design and is actively being updated. Some features described as "long term" have already been implemented.

## 1. System Directory & Workspace Crate Architecture

To ensure separation of concerns, compile-time memory optimization, and strict safety boundaries between the high-level planning host (klipper-host) and the bare-metal microcontroller target (klipper-mcu-firmware), the workspace is organized as follows:

```
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
│   │       ├── stepper.rs      <-- Step timers and registers
│   │       └── stepper_dma.rs  <-- DMA-based step engine
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
```

## 2. Phase 1 (Short Term) Specifications

### Task 1.1: Dynamic SPSC Lock-Free Step Queue
- **Target Crate**: `mcu-drivers`
- **Target File**: `crates/mcu-drivers/src/stepper.rs`
- **Status**: Implemented.

### Task 1.2: Self-Describing Pinout Configuration Protocol
- **Target Crate**: `klipper-proto`
- **Target File**: `crates/klipper-proto/src/autoconfig.rs`
- **Status**: Implemented.

### Task 1.3: Rhai Scripting VM Integration
- **Target Crate**: `klipper-host`
- **Target File**: `crates/klipper-host/src/macro_engine.rs`
- **Status**: Implemented.

## 3. Phase 2 (Medium Term) Specifications

### Task 2.1: Degree-15 Pythagorean-Hodograph (PH) Corner Blending Solver
- **Target Crate**: `motion`
- **Target File**: `crates/motion/src/ph_beziers.rs`
- **Status**: Implemented.

### Task 2.2: G^4 Motion Profile (31-Phase Trajectory Generator)
- **Target Crate**: `motion`
- **Target File**: `crates/motion/src/g4_planner.rs`
- **Status**: Implemented.

### Task 2.3: State-Space MPC Thermal Control Module
- **Target Crate**: `thermal`
- **Target File**: `crates/thermal/src/mpc.rs`
- **Status**: Implemented.

### Task 2.4: Overshoot-Constrained Homing Math Module
- **Target Crate**: `motion`
- **Target File**: `crates/motion/src/homing.rs`
- **Status**: Implemented.

## 4. Phase 3 (Long Term) Specifications

### Task 3.1: Multi-MCU Clock Synchronization (RTIC 2 Clock Sync)
- **Target Crate**: `klipper-mcu-firmware`
- **Target File**: `crates/klipper-mcu-firmware/src/clock_sync.rs`
- **Status**: Implemented.

### Task 3.2: Hardware Timer + DMA Step Offloading
- **Target Crate**: `mcu-drivers`
- **Target File**: `crates/mcu-drivers/src/stepper_dma.rs`
- **Status**: Implemented.

### Task 3.3: Dynamic HIL Benchmarking & Noise Profiling
- **Target Crate**: `klipper-host`
- **Target File**: `crates/klipper-host/src/hil_analyzer.rs`
- **Status**: Implemented.
