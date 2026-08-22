# `r_klipp` Product Requirements Document (PRD)

This document specifies the product requirements, functional & non-functional specifications, multi-domain machine targets, release milestones, and validation criteria for **`r_klipp`**.

---

## 🎯 1. Product Scope & Vision

**`r_klipp`** is a high-performance, real-time, memory-safe machine operating system and motion control engine written in Rust. It serves as a unified control plane across three major machine classes:
1. **High-Speed Additive Manufacturing (3D Printing)**
2. **Surface-Mount Pick & Place (PnP / PIP)**
3. **Multi-Axis Subtractive CNC Machining (3–5 Axis Mills)**

---

## 📋 2. Functional Requirements by Machine Domain

### 2.1 3D Printing Domain
- **FR-3DP-01**: Cartesian, CoreXY, and Delta kinematics transformations.
- **FR-3DP-02**: $G^4$ 31-phase trajectory planning with continuous jerk, snap, and crackle bounds.
- **FR-3DP-03**: Pressure Advance extrusion compensation ($k \cdot v$) and volumetric multipliers.
- **FR-3DP-04**: State-space Model Predictive Control (MPC) with Kalman filtering and extrusion feedforward.

### 2.2 Pick & Place (PnP / PIP) Domain
- **FR-PNP-01**: Dual-head kinematics ($Z_1, Z_2, C_1, C_2$) with dynamic toolhead clearance boundaries.
- **FR-PNP-02**: Position-synchronized discrete hardware I/O for sub-millisecond bottom-camera exposure pulses.
- **FR-PNP-03**: CAN-FD Smart Feeder RPC protocol for tape advancement, peeling, and telemetry.
- **FR-PNP-04**: TMC2240 StallGuard torque surveillance for solder paste and adhesive dispensing.
- **FR-PNP-05**: Direct bidirectional integration bridge with OpenPnP.

### 2.3 3–5 Axis CNC Domain
- **FR-CNC-01**: 5-Axis Table-Table AC Trunnion kinematics with Rotary Tool Center Point (RTCP) preservation.
- **FR-CNC-02**: VFD spindle speed controller with Constant Surface Speed (CSS / G96).
- **FR-CNC-03**: Dynamic G43/G44 tool length offset and radius wear compensation table.
- **FR-CNC-04**: Single-tick G38.2 contact probing supervisor and enclosure door safety interlocks.

---

## ⚡ 3. Non-Functional Requirements (NFRs)

- **NFR-01 (Memory Safety)**: All real-time motion and firmware crates must run `#![no_std]` with zero heap allocations in critical execution loops.
- **NFR-02 (Step Jitter)**: Hardware step pulse jitter shall remain $< 100\text{ ns}$ under full load.
- **NFR-03 (Clock Synchronization)**: Multi-MCU clock skew shall remain $< 1\,\mu\text{s}$ via continuous DPLL tracking.
- **NFR-04 (Safety Response)**: Watchdog expiration, thermal runaway, or door breach must cut power to heaters, spindle, and steppers in $< 100\text{ ms}$.

---

## 🚀 4. System Validation & Verification Metrics

| Domain | Test Metric | Acceptance Threshold |
| :--- | :--- | :--- |
| **Motion Planner** | Property Tests & Determinism | 100% bit-identical trajectories across runs |
| **DPLL Sync** | Multi-MCU Timer Jitter | $< 1\,\mu\text{s}$ relative skew over 1-hour test |
| **Thermal Engine** | Steady-State Variance | $< \pm 0.1^\circ\text{C}$ at $250^\circ\text{C}$ setpoint |
| **PnP Camera Sync** | Shutter Pulse Precision | $\pm 10\,\mu\text{s}$ spatial trigger accuracy |
| **CNC Probing** | G38.2 Contact Deceleration | Latch within 1 timer tick, zero probe overtravel |
