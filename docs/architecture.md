# `r_klipp`: System Architecture & Multi-Domain Engineering

This document provides a comprehensive technical breakdown of the **`r_klipp`** machine operating system, covering design principles, numeric abstractions, concurrency models, multi-domain kinematics, distributed DPLL clock synchronization, and host-MCU communication.

---

## 🏛️ 1. Core Architectural Tenets

1. **Bare-Metal Memory Safety (`no_std`)**:
   - The core motion planner, kinematics engine, parser, safety supervisor, and protocol codecs compile under `#![no_std]` with zero dynamic heap allocations in execution paths.
2. **Universal Translator Architecture**:
   - The system decouples motion planning from physical machine mechanics via universal HAL traits and pluggable kinematics modules supporting **3D Printers**, **Pick & Place (PnP / PIP)**, and **3–5 Axis CNC Mills**.
3. **Dual-Paradigm Concurrency**:
   - **Cooperative Multitasking**: Asynchronous I/O, telemetry streams, and state updates run cooperatively under async executors (Embassy on embedded, Tokio on host).
   - **Preemptive Real-Time Interrupts**: Microsecond pulse timing, timer reloads, and input captures execute deterministically via hardware interrupt priority controllers (RTIC 2 / ARM NVIC).
4. **Sub-Microsecond Multi-MCU DPLL Synchronization**:
   - Real-time hardware timers on toolhead boards and mainboards are disciplined against a central clock model via a continuous 2nd-order digital phase-locked loop.

---

## 🌐 2. System Architecture Topology

```
+-----------------------------------------------------------------------------------------+
|                                    HOST APPLICATION LAYER                               |
|                                                                                         |
|  +-----------------------+     +-----------------------+     +-----------------------+  |
|  |     host-ui           |     |     OpenPnP Studio    |     |   Fluidd / Mainsail   |  |
|  |  (Slint 1.x Touch UI) |     |  (G-Code TCP Bridge)  |     |  (Moonraker JSON-RPC) |  |
|  +-----------------------+     +-----------------------+     +-----------------------+  |
+--------------------------------------------|--------------------------------------------+
                                             v
+-----------------------------------------------------------------------------------------+
|                                   HOST SERVER RUNTIME                                   |
|                                                                                         |
|  +-----------------------+     +-----------------------+     +-----------------------+  |
|  |   Actix-Web REST/WS   | <-> |  Embedded SurrealDB   | <-> |  Serial/CAN-FD Bridge |  |
|  |   API & Moonraker     |     |  (Telemetry & Files)  |     |  (COBS + Postcard)    |  |
|  +-----------------------+     +-----------------------+     +-----------------------+  |
+--------------------------------------------|--------------------------------------------+
                                             v (USB CDC-ACM / CAN-FD / RS485)
+-----------------------------------------------------------------------------------------+
|                                    REAL-TIME CORE (no_std)                              |
|                                                                                         |
|  +-----------------------------------------------------------------------------------+  |
|  |                                  G-Code Parser                                    |  |
|  +-----------------------------------------------------------------------------------+  |
|                                            |                                            |
|                                            v                                            |
|  +-----------------------------------------------------------------------------------+  |
|  |               G4 31-Phase Trajectory Generator & PH Corner Blender                |  |
|  +-----------------------------------------------------------------------------------+  |
|         |                                  |                                 |          |
|         v (3D Printing)                    v (Pick & Place)                  v (CNC)    |
|  +-----------------------+     +-----------------------+     +-----------------------+  |
|  | CoreXY / Delta / Cart |     | Dual-Head PnP Kinemat |     | 5-Axis Table RTCP     |  |
|  | Pressure Advance E    |     | Discrete Sync I/O     |     | VFD CSS Spindle (G96) |  |
|  | Kalman MPC Thermal    |     | CAN-FD Smart Feeders  |     | G43 Tool Offset Table |  |
|  +-----------------------+     +-----------------------+     +-----------------------+  |
|                                            |                                            |
|                                            v                                            |
|  +-----------------------------------------------------------------------------------+  |
|  |                           Lock-Free SPSC Step Queue                               |  |
|  +-----------------------------------------------------------------------------------+  |
|                                            |                                            |
|                                            v                                            |
|  +-----------------------------------------------------------------------------------+  |
|  |                   Universal HAL & DMA Hardware Pulse Generator                    |  |
|  +-----------------------------------------------------------------------------------+  |
+-----------------------------------------------------------------------------------------+
```

---

## 📐 3. Multi-Domain Kinematics & Subsystems

### 3.1 3D Printer Subsystem
- **$G^4$ 31-Phase Trajectory Generator**: Limits crackle, snap, jerk, and acceleration across continuous mathematical phases to eliminate resonance.
- **Pressure Advance Modeling**: Dynamic advance correction:
  $$E_{\text{comp}}(t) = k_{\text{pa}} \cdot v(t)$$
- **Kalman-Filtered State-Space MPC**: Controls hotend and heated bed power with predictive flow-rate feedforward.

### 3.2 Pick & Place (PnP / PIP) Subsystem
- **`DualHeadPnpKinematics`**: Controls dual vertical Z-axes ($Z_1, Z_2$) and dual rotary C-axes ($C_1, C_2$) with hardware clearance boundary checking.
- **`SyncIoScheduler`**: Emits microsecond camera exposure strobes and vacuum sequencing synchronized with toolhead spatial positions.
- **CAN-FD Smart Feeders**: RPC interface for tape feeding, pitch calibration, and part tape peeling.

### 3.3 3–5 Axis CNC Subsystem
- **`FiveAxisRtcpKinematics`**: 5-Axis Table-Table AC Trunnion kinematics with Rotary Tool Center Point preservation.
- **`SpindleController`**: Dynamic VFD speed regulation with Constant Surface Speed (CSS / G96).
- **`ToolTableManager`**: Dynamic G43/G44 tool length offsets and tool wear compensation.
- **`ProbeSupervisor` & `EnclosureInterlock`**: Single-tick G38.2 contact latching and high-speed spindle door interlocks.

---

## ⏱️ 4. Multi-MCU Distributed Phase-Locked Loop (DPLL)

The DPLL clock synchronizer maps local hardware timers $T_{\text{mcu}}$ to master coordinates $T_{\text{master}}$:

$$T_{\text{master}}(t) = \alpha \cdot T_{\text{mcu}}(t) + \beta$$

- Continuous recursive least squares updates at $10\text{ Hz}$.
- Rejects transmission outliers where roundtrip latency exceeds normal statistical thresholds.
- Guarantees multi-board synchronized step execution within $< 1\,\mu\text{s}$.
