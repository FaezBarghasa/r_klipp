# Changelog

All notable changes to the `r_klipp` project are documented in this file. This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.6.0] - 2026-08-22

### 🚀 Universal Machine Control Engine & Multi-Target Architecture
This major release establishes `r_klipp` as a domain-agnostic machine operating system spanning **3D Printing**, **Surface-Mount Pick & Place (PnP / PIP)**, and **3–5 Axis CNC Machining**.

### Added
- **Universal Hardware Abstraction Layer (`crates/hal/src/universal.rs`)**:
  - Implemented domain-agnostic traits: `StepperAxis`, `DigitalOutput` (with `scheduled_set_state`), `AnalogInput`, `PwmOutput`, and `CommunicationBus`.
- **Target Profiles & Schema Validation (`crates/compat-layer/src/profiles.rs`)**:
  - Multi-target profiles (`ThreeDPrinter`, `PnpPip`, `Cnc`) with template configurations in `profiles/3d_printer.toml`, `profiles/pnp.toml`, `profiles/cnc.toml`.
- **Target 1: 3D Printing Baseline**:
  - Advanced Pressure Advance modeling ($k \cdot v$) and volumetric extrusion multiplier calculations (`crates/motion/src/extruder.rs`).
- **Target 2: Pick & Place (PnP / PIP) Capabilities**:
  - `DualHeadPnpKinematics` (`crates/kinematics/src/pnp_dual_head.rs`) with independent Z1/Z2 vertical and C1/C2 theta axes.
  - `SyncIoScheduler` (`crates/motion/src/sync_io.rs`) for sub-millisecond bottom-camera exposure triggering.
  - CAN-FD Smart Feeder RPC protocol (`crates/klipper-proto/src/feeder.rs`).
  - TMC2240 StallGuard torque surveillance for solder paste dispensing (`crates/motion/src/stall_guard.rs`).
  - OpenPnP G-Code and HTTP bridge (`host-server/src/openpnp.rs`).
- **Target 3: 3–5 Axis CNC Capabilities**:
  - `SpindleController` (`crates/motion/src/spindle.rs`) with VFD PWM and Constant Surface Speed (CSS / G96).
  - `FiveAxisRtcpKinematics` (`crates/kinematics/src/rtcp.rs`) for 5-Axis Table-Table AC Trunnion RTCP tool tip preservation.
  - `ToolTableManager` (`crates/motion/src/tool_compensation.rs`) with G43/G44 length and radius wear offsets.
  - `ProbeSupervisor` and `EnclosureInterlock` (`crates/safety/src/probe_interlock.rs`) for single-tick G38.2 contact latching and spindle door safety stops.
- **Host Server & Touch UI**:
  - Modern Slint 1.x touch screen console in `host-ui`.
  - Multi-threaded Moonraker/OpenPnP host server backed by embedded SurrealDB in `host-server`.

---

## [0.5.1] - 2026-06-02
- Maintenance release modernizing dependencies and CI checks.

---

## [0.5.0] - 2026-06-02
- Degree-15 Pythagorean-Hodograph (PH) corner blending.
- $G^4$ 31-Phase high-order trajectory generator.
- State-space MPC thermal regulation with Kalman filtering.
- Distributed Phase-Locked Loop (DPLL) clock synchronization.
