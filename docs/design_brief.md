# r_klipp Design Brief Document

This document outlines the product vision, core design philosophy, architectural goals, target user personas, technical requirements, and interface aesthetic guidelines for **r_klipp**.

---

## 1. Vision & Executive Summary

**r_klipp** is a next-generation, high-performance 3D printer and CNC motion control system developed from the ground up in Rust. Designed as a zero-cost abstraction, high-reliability alternative to legacy C/Python-based Klipper firmware, `r_klipp` brings bare-metal safety, mathematical determinism, state-space thermal regulation, and sub-microsecond multi-MCU synchronization to modern additive and subtractive manufacturing hardware.

---

## 2. Core Design Principles

1. **Bare-Metal Memory Safety (`no_std`)**:
   - Firmware runs entirely without OS or dynamic heap allocators (`alloc`).
   - All buffers are static, stack-allocated, or use bounded `heapless` collections, eliminating heap fragmentation, buffer overflows, and pointer corruption.
2. **Dual-Paradigm Concurrency**:
   - **Cooperative Multitasking**: Non-blocking I/O, telemetry, serial communications, and host commands are orchestrated using the async/await paradigm of the **Embassy executor**.
   - **Preemptive Real-Time Interrupts**: Step generation, hardware timer reloads, and time-critical pulse capturing run under high-priority hardware interrupts managed by **RTIC 2**.
3. **Numeric Determinism**:
   - Floating-point calculations are replaced with fixed-point math (`Fixed16_16` saturating arithmetic) on the MCU to guarantee microsecond-level timing consistency across all MCU architectures.
4. **Hardware Agnosticism & Modular HAL**:
   - Strict separation between logic crates (`r_klipp_motion`, `r_klipp_thermal`, `r_klipp_safety`) and hardware drivers using `embedded-hal` traits, allowing effortless porting between STM32, RP2040, i.MX RT, and RISC-V targets.

---

## 3. Target User Personas

| Persona | Key Needs & Pain Points | How r_klipp Delivers |
| :--- | :--- | :--- |
| **High-Speed 3D Printing Enthusiast** | Wants maximum printing speed, ultra-smooth surface finishes, and input shaping without motor stuttering or firmware crashes. | Degree-15 PH corner blending, G4 31-phase trajectory planning, and 1 MHz DMA stepper engine. |
| **Industrial CNC & Machine Builder** | Requires absolute reliability, precise homing protection, multi-axis synchronization, and multi-board setups. | Sub-microsecond DPLL multi-MCU clock sync, braking curve homing, and compile-time memory safety. |
| **Embedded Firmware Engineer** | Seeks modular, maintainable code, self-describing hardware configuration, and modern Rust tooling. | Self-describing `HandshakeManifest` autoconfig protocol, zero-copy `postcard` IPC, and standard Cargo workspace. |

---

## 4. Technical Requirements & Performance Benchmark Goals

- **Step Generation Rate**: Up to **1,000,000 steps per second (1 MHz)** per axis via DMA double-buffering without CPU timing jitter.
- **Multi-MCU Synchronization**: Maintain clock skew below **100 nanoseconds** across multiple secondary MCUs using Distributed Phase-Locked Loop (DPLL) linear regression.
- **Autoconfig Negotiation**: Complete hardware discovery and configuration handshake within **1.5 seconds** of serial connection.
- **Corner Blending Continuity**: Maintain continuous $C^4$ jerk/snap continuity along toolpaths using 15th-degree Pythagorean-Hodograph Bezier curves.
- **Thermal Control Precision**: Maintain nozzle and heatbed temperature stability within **$\pm 0.1^\circ\text{C}$** under variable extrusion loads via state-space Kalman filter MPC.

---

## 5. System Interface & Experience Design

### 5.1. Host CLI & Web UI Aesthetics
- **Design Paradigm**: Clean, futuristic, high-density industrial control panel with sleek dark mode palette (deep slate `#0d1117`, cyan accents `#58a6ff`, emerald status `#3fb950`, coral alert `#f85149`).
- **Real-Time Data Visualizations**: High-frequency canvas graphing for state-space thermal tracking, real-time input shaper FFT frequency responses, and step-queue saturation meters.
- **Responsive Telemetry**: Sub-50ms WebSocket telemetry streaming with low overhead binary data frames.

### 5.2. Developer Experience (DX)
- Clear Cargo workspace structure splitting binary targets (`r_klipp_host`, `r_klipp_firmware`) and library crates (`r_klipp_api`, `r_klipp_motion`, `r_klipp_thermal`, `r_klipp_safety`).
- Automated HIL (Hardware-in-the-Loop) test suites and simulation frameworks (`sim` crate) for validating kinematic planning without physical hardware.
