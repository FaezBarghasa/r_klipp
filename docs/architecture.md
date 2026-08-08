# Klipper in Rust: System Architecture

This document provides a low-level, comprehensive overview of the architecture of the **r_klipp** firmware and host system. It outlines the design principles, numeric abstractions, component interactions, concurrency models, and detailed subsystems of the codebase.

---

## 1. Core Architectural Principles

The `r_klipp` system is built on the following foundational tenets:

1. **Bare-Metal Type & Memory Safety (`no_std`)**: The firmware runs entirely without an operating system or heap allocator. Memory safety is enforced at compile time using static, stack-allocated, and `heapless` data structures, eliminating memory fragmentation, buffer overflows, and null-pointer dereferences.
2. **Dual-Paradigm Concurrency**:
   - **Cooperative Multitasking**: High-level, non-blocking I/O operations (such as serial communication, monitoring, and state updates) are executed cooperatively using the async/await framework of the **Embassy executor**.
   - **Preemptive Real-Time Interrupts**: Extremely time-critical operations (such as step pulse generation, hardware timer reloads, and clock synchronization captures) are scheduled using hardware interrupt priorities under the **RTIC 2** framework.
3. **Numeric Determinism**: All floating-point and algebraic motion math are either modeled offline or executed on the MCU using strict fixed-point representations to guarantee identical step timings and prevent software FPU bottlenecks or non-deterministic execution times.
4. **Modularity & HAL Boundary**: Code is structured into isolated Cargo crates with clear, hardware-agnostic interfaces. Subsystems talk to the hardware abstraction layer (HAL) through traits defined by `embedded-hal` and `embedded-hal-nb`, simplifying porting.

---

## 2. System Architecture Overview

The system is split between the high-level orchestration host (`klipper-host`) and the target MCU firmware (`klipper-mcu-firmware`).

```
 +-----------------------------------------------------------------------------------+
 |                                   KLIPPER HOST                                    |
 |                                                                                   |
 |  +-----------------------+     +-----------------------+     +-----------------+  |
 |  |    G-Code Parser      | --> |  Host Macro Engine    | --> |  HIL Analyzer   |  |
 |  |                       |     |  (Rhai Scripting VM)  |     |  (Input Shaper) |  |
 |  +-----------------------+     +-----------------------+     +-----------------+  |
 |              |                                                        |           |
 |              v                                                        v           |
 |  +-----------------------+                                   +-----------------+  |
 |  |  Kinematic Planner    |                                   |  Postcard Ser/  |  |
 |  |  (PH & G4 Trajectory) | --------------------------------> |  Deserializer   |  |
 |  +-----------------------+                                   +-----------------+  |
 +-----------------------------------------------------------------------|-----------+
                                                                         | (UART/USB)
                                                                         v
 +-----------------------------------------------------------------------------------+
 |                               MCU FIRMWARE TARGET                                 |
 |                                                                                   |
 |  +-----------------------+     +-----------------------+     +-----------------+  |
 |  |  Proto Bridge / Rx    | --> |  Command Dispatcher   | <-- |   Clock Sync    |  |
 |  |  (Postcard De-ser)    |     |                       |     |   (DPLL Model)  |  |
 |  +-----------------------+     +-----------------------+     +-----------------+  |
 |              |                             |                          |           |
 |              v                             v                          v           |
 |  +-----------------------+     +-----------------------+     +-----------------+  |
 |  |  Step Queue (SPSC)    |     |  MpcThermalEngine     |     |  SafetyMonitor  |  |
 |  |  (StepSegment array)  |     |  (Kalman State Space) |     |  (Watchdog/Est) |  |
 |  +-----------------------+     +-----------------------+     +-----------------+  |
 |              |                             |                          |           |
 |              v                             v                          v           |
 |  +-----------------------+     +-----------------------+     +-----------------+  |
 |  |  Stepper/DMA Engine   |     |  PWM / ADC Drivers    |     | Heaters/Motors  |  |
 |  |  (DmaStepEngine)      |     |                       |     |  Failsafe Pins  |  |
 |  +-----------------------+     +-----------------------+     +-----------------+  |
 +-----------------------------------------------------------------------------------+
```

Detailed representations of this architecture, interface interactions, and scheduling models can be found in [docs/diagram.md](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/docs/diagram.md).

---

## 3. Core Subsystems & Components

### 3.1. Numeric Representation (`Fixed16_16`)
To achieve high-speed computation without floating-point units, the firmware uses a custom wrapper `Fixed16_16` located in [fixed_point.rs](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/crates/klipper-mcu-firmware/src/fixed_point.rs).
- **Core Engine**: Wraps `fixed::types::I16F16` from the `fixed` crate.
- **Overrun Safety**: Every arithmetic operation (`Add`, `Sub`, `Mul`, `Div`, `Neg`) is implemented using **saturating operations** (e.g., `saturating_add`, `saturating_mul`). Overflows and underflows cap at maximum or minimum limits rather than wrapping or causing runtime panics.

### 3.2. Host Scripting & Macro Engine
To support G-Code macro configuration dynamically at runtime, `klipper-host` includes the `HostMacroEngine` located in [macro_engine.rs](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/crates/klipper-host/src/macro_engine.rs).
- **Rhai VM**: Integrates a sandboxed Rhai script engine.
- **Starvation Protection**: Caps execution steps via `engine.set_max_operations(100_000)` to prevent infinite loops from locking the main scheduler threads.
- **Pipeline Integration**: Injects G-Code commands directly back into the planning channel in a thread-safe manner using `std::sync::mpsc::Sender`.

### 3.3. Host-MCU Communication & Autoconfig Protocol
Communication is implemented inside the `klipper-proto` crate:
- **Self-Describing Pinout Config**: During startup, the MCU compiles a detailed hardware manifest (`HandshakeManifest` in [autoconfig.rs](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/crates/klipper-proto/src/autoconfig.rs)) listing all GPIO index mappings, names, and a list of `PinCapability` configurations (e.g. `DigitalInput`, `DigitalOutput { max_current_ma }`, `PwmOutput { max_freq_hz }`).
- **Postcard Serialization**: The manifest is serialized using the `postcard` protocol for compact zero-copy transfers, ensuring negotiation occurs within 1.5 seconds of serial connection.

### 3.4. Advanced Motion Planning Subsystem
The motion system is located in the `motion` and `mcu-drivers` crates:
- **SPSC Lock-Free Step Queue**: Inside [stepper.rs](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/crates/mcu-drivers/src/stepper.rs), `StepperController` holds a lock-free Single-Producer Single-Consumer (SPSC) ring buffer (`heapless::spsc::Queue`) containing `StepSegment` structures. The communications task fills the queue, and the timer interrupts drain it.
- **Degree-15 Pythagorean-Hodograph (PH) Corner Blending**: Implemented in [ph_beziers.rs](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/crates/motion/src/ph_beziers.rs), `PhBezier15` calculates continuous $C^4$ bezier curves to bridge linear movements. It uses Kahan compensated summation for arc-length evaluation and Newton-Raphson iterations to resolve parameter convergence down to $<10^{-9}$ mm.
- **G^4 31-Phase Trajectory Generator**: Located in [g4_planner.rs](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/crates/motion/src/g4_planner.rs), `G4Profile` calculates movement profiles restricting crackle, snap, jerk, and acceleration across 31 discrete kinematic phases.
- **DMA Offloading Engine**: Inside [stepper_dma.rs](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/crates/mcu-drivers/src/stepper_dma.rs), `DmaStepEngine` manages double-buffered memory blocks (`buffer_a` and `buffer_b`) containing stepping pulse interval timings. This allows the MCU to sustain up to a 1 MHz step rate with zero CPU jitter by natively offloading pulse generation.
- **Braking Curve Guard Homing**: Implemented in [homing.rs](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/crates/motion/src/homing.rs), `SafeHomingPlanner` scales homing velocity based on deceleration capabilities and limits physical stopping distances under an 85% safety factor.

### 3.5. State-Space MPC Thermal Subsystem
Heater control is governed by the state-space Model Predictive Control (MPC) engine in [mpc.rs](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/crates/thermal/src/mpc.rs).
- **Kalman Filtering State Estimation**: Establishes discrete matrices to track estimated sensor temperature $T_{sensor}$ and internal heater core temperature $T_{heater}$. Step updates execute a prediction model using state variables ($a, b, g$) and perform corrections against measured ADC inputs.
- **Feed-Forward Control**: Calculates PWM power by scaling current ahead of volumetric filament extrusion changes, preventing temperature drops during high-speed printing.

### 3.6. Multi-MCU Clock Synchronization
To coordinate stepper timing across multiple independent MCUs, the system features a Distributed Phase-Locked Loop (DPLL) clock sync mechanism in [clock_sync.rs](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/crates/klipper-mcu-firmware/src/clock_sync.rs).
- **Linear Regression Fitting**: Computes slope ($m$) and intercept ($c$) coefficients for $y = mx + c$ tick conversions using recursive least squares regression.
- **Lock-Free Sharing**: `SharedClockModel` provides atomic double-buffering. Low-priority tasks update the inactive model and atomically swap it, enabling high-priority NVIC Priority 4 interrupts to query model parameters without locking or blocking.

### 3.7. Safety & Supervision Subsystem
Safety monitoring is implemented in `safety.rs` in [safety.rs](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/crates/klipper-mcu-firmware/src/safety.rs).
- **Hardware Watchdog**: Continuously pets the independent watchdog (IWDG).
- **Failsafe Monitoring**: Evaluates sensor limits, rate-of-change thermal runaways, task check-in deadlines, and stepper driver fault masks.
- **Atomic Emergency Stop**: Any safety violation atomically transitions the system status to a shutdown state, immediately disabling heater PWM and stepper driver enable pins.

---

## 4. Task Scheduling & Concurrency Model

Task scheduling leverages **Embassy** for asynchronous task polling and **RTIC 2** for priority-driven hardware interrupts.

| Task / Interrupt | Priority | Executor / Framework | Description |
|------------------|----------|----------------------|-------------|
| **Stepper Timer ISR** | Priority 5 | Hardware ISR | Reloads timer registers directly from the step queue. |
| **Clock Sync Capture**| Priority 4 | Hardware ISR | Captures raw timers and reads the active clock regression model. |
| **Serial Rx/Tx** | Priority 3 | Embassy / DMA | Reads UART/USB buffers, runs zero-copy packet parses. |
| **Thermal Update** | Priority 2 | Embassy Task | Samples ADC channels, updates Kalman filter, and adjusts PWM. |
| **Clock Model Task** | Priority 1 | Embassy Task | Processes history arrays and updates clock models. |
| **Safety Supervision**| Priority 1 | Embassy Task | Checks watchdog status and monitors check-in deadlines. |

### Tasks Communication Protocols

Subsystems share data using thread-safe, lock-free primitives:
- **`embassy-sync` Channels**: Used for message-passing between tasks (e.g. command parsing forwarding events).
- **Atomic Flags**: Provide global, interrupt-safe status flags (e.g. emergency stop triggers).
- **Double-Buffered Structs**: Swap pointers under atomic loads/stores to share configuration metrics (e.g. clock model updates).
