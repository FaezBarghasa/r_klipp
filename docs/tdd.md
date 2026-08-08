# r_klipp Technical Design Document (TDD)

This document details the software architecture, module organization, concurrency paradigms, mathematical algorithms, hardware integration layers, and fault-tolerance mechanics of **r_klipp**.

---

## 1. System Architecture & Cargo Workspace Structure

`r_klipp` is architected as a modular Cargo workspace, separating high-level orchestration, IPC contracts, motion planning, control theory, hardware abstraction, and target firmware executables.

```
r_klipp/
├── crates/
│   ├── bin/
│   │   ├── r_klipp_host          # High-level Linux host service (G-Code orchestration, Rhai VM)
│   │   └── r_klipp_firmware      # Target MCU embedded binary (no_std, Embassy + RTIC 2)
│   ├── libs/
│   │   ├── r_klipp_api           # Shared API data models and telemetry schemas
│   │   ├── r_klipp_motion        # Motion planning, trajectory generators, PH Beziers
│   │   ├── r_klipp_thermal       # State-space MPC heater control & Kalman filters
│   │   └── r_klipp_safety        # Hardware safety watchdog & error handling engine
│   ├── klipper-proto             # Binary serialization (postcard) & autoconfig protocol
│   ├── mcu-drivers               # Stepper DMA engines, PWM, ADC, and timer drivers
│   ├── kinematics                # Kinematic models (Cartesian, CoreXY, Delta, SCARA)
│   └── parser                    # G-Code lexer, parser, and AST generator
```

---

## 2. Concurrency Architecture & Runtime Model

The MCU firmware target (`r_klipp_firmware`) uses a hybrid dual-paradigm concurrency model combining **Embassy** (cooperative async/await) and **RTIC 2** (preemptive priority-based interrupts):

```
+-----------------------------------------------------------------------------------+
|                              MCU FIRMWARE RUNTIME                                 |
|                                                                                   |
|  +-----------------------------------------------------------------------------+  |
|  | HARDWARE INTERRUPTS (RTIC 2 - Preemptive Priority Execution)                |  |
|  |                                                                             |  |
|  |   Priority 3 (Highest): Timer Reload & Stepper DMA Engine Interrupt        |  |
|  |   Priority 2          : Endstop Trigger & Safety Hardware Trip Intercept   |  |
|  |   Priority 1          : DPLL Hardware Timestamp Capture Capture            |  |
|  +-----------------------------------------------------------------------------+  |
|                                         |                                         |
|                                     Shared SPSC                                   |
|                                    Lock-Free Queues                               |
|                                         v                                         |
|  +-----------------------------------------------------------------------------+  |
|  | ASYNC COOPERATIVE EXECUTOR (Embassy Task Scheduler)                         |  |
|  |                                                                             |  |
|  |   Task 1: USB / Serial Transport Rx/Tx (postcard deserialization)          |  |
|  |   Task 2: State-Space MPC Thermal Estimation & PWM Update (10 Hz)          |  |
|  |   Task 3: Telemetry Streamer & Periodic Health Check (50 Hz)                |  |
|  +-----------------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------------+
```

---

## 3. Advanced Kinematics & Motion Planning Subsystem

### 3.1. Degree-15 Pythagorean-Hodograph (PH) Corner Blending
To eliminate abrupt jerk discontinuities when transitioning between linear G-Code segments without sacrificing speed, `r_klipp_motion` computes continuous $C^4$ Bezier curves:

- **Mathematical Form**: Computes 15th-degree PH Bezier curve segments $r(u)$ bridging incoming vector $\mathbf{v}_1$ and outgoing vector $\mathbf{v}_2$.
- **Arc-Length Integration**: Evaluates curve length using **Kahan compensated summation** to eliminate floating-point truncation error.
- **Newton-Raphson Parameter Mapping**: Resolves distance-to-parameter $u(s)$ conversion iteratively down to sub-nanometer error bounds ($< 10^{-9}\text{ mm}$).

### 3.2. G4 31-Phase Trajectory Generator (`G4Profile`)
Generates motion profiles restricting crackle ($d^5 x / dt^5$), snap ($d^4 x / dt^4$), jerk ($d^3 x / dt^3$), and acceleration ($d^2 x / dt^2$):
- Divides each movement into up to 31 discrete kinematic phases.
- Guarantees zero residual vibration at motion segment termination.

### 3.3. DMA Stepper Pulse Offloading Engine (`DmaStepEngine`)
- Stepper pulse generation is offloaded directly to hardware DMA channels tied to timer compare registers.
- Double-buffered RAM blocks (`buffer_a` and `buffer_b`) hold pre-calculated pulse timing arrays.
- Eliminates CPU interrupt overhead per step pulse, sustaining step rates up to **1 MHz**.

---

## 4. State-Space MPC Thermal Control Subsystem

Rather than conventional PID controllers which suffer from thermal lag and overshoot, `r_klipp_thermal` implements state-space Model Predictive Control with Kalman filtering:

1. **State Vector Estimation**:
   The internal state vector $\mathbf{x} = \begin{bmatrix} T_{heater} \\ T_{sensor} \end{bmatrix}^T$ models heater core and thermistor temperatures independently.
2. **Kalman Prediction & Correction**:
   Predicts next temperature states based on current PWM power input $u$, then applies correction gain $\mathbf{K}$ against measured ADC input:
   $$\hat{\mathbf{x}}_{k+1} = \mathbf{A} \hat{\mathbf{x}}_k + \mathbf{B} u_k + \mathbf{K} (y_{measured} - \mathbf{C} \hat{\mathbf{x}}_k)$$
3. **Feed-Forward Volumetric Extrusion Scaling**:
   Calculates required thermal energy in advance based on filament extrusion volume rate ($\text{mm}^3/\text{s}$), holding temperatures rock-solid during abrupt speed changes.

---

## 5. Multi-MCU Clock Synchronization Subsystem

To synchronize motor execution across multiple distinct microcontroller boards:
- **DPLL Linear Regression**: The host fits a recursive least-squares line $y = mx + c$ mapping host clock ticks to target MCU timer ticks.
- **Microsecond Jitter Mitigation**: MCU timer capture interrupts lock incoming sync packets, adjusting step scheduling delays on the fly to eliminate drift across axes.

---

## 6. Safety Monitoring & Fault-Tolerance Architecture

`r_klipp_safety` runs continuous diagnostic sweeps:
- **Hardware Watchdog**: Reset periodically by the Embassy executor. If execution stalls for $> 500\text{ ms}$, the hardware watchdog trips and forces safe MCU reset.
- **Braking Curve Guard Homing**: Verifies homing speeds against current axis position to ensure physical stopping distance never exceeds available endstop over-travel ($85\%$ safety margin).
- **Thermal Boundary Monitoring**: Immediately trips emergency shutdown if $T_{sensor}$ deviates from $T_{heater}$ model prediction by more than $\pm 15^\circ\text{C}$ for $> 2.0\text{ seconds}$.
