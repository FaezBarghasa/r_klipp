# `r_klipp_motion`: High-Order Trajectory Planning & Multi-Domain Motion Control

`r_klipp_motion` is a `no_std`, real-time motion planning engine providing jerk-limited trajectory generation, continuous corner blending, extruder pressure advance, synchronized discrete I/O, spindle regulation, and tool compensation.

---

## 🚀 Advanced Capabilities

### 1. High-Order Trajectory Generation ($G^4$ 31-Phase Profile)
- Provides continuous jerk ($j(t)$), snap ($s(t)$), and crackle ($c(t)$) bounds.
- Eliminates mechanical resonance, chassis vibration, and belt ringing at high printing/milling speeds.

### 2. Pythagorean-Hodograph (PH) Corner Blending
- Solves $C^4$ continuous Bezier polynomial bridge curves between linear segments using Newton-Raphson parameter solvers.
- Maintains high cornering velocities without violating centripetal acceleration limits.

### 3. Extruder Modeling & Pressure Advance (`extruder.rs`)
- Implements linear pressure advance compensation:
  $$E_{\text{comp}}(t) = k_{\text{pa}} \cdot v(t)$$
- Volumetric multiplier scaling and non-linear flow rate corrections.

### 4. Position-Synchronized Discrete I/O (`sync_io.rs`)
- Microsecond-accurate digital I/O event scheduler.
- Directly triggers bottom-vision cameras, lighting strobes, and vacuum pick/place solenoids at exact spatial coordinates during motion.

### 5. Closed-Loop Spindle & Constant Surface Speed (`spindle.rs`)
- VFD PWM and Modbus RS485 spindle speed regulation.
- Implements G96 Constant Surface Speed (CSS): dynamically adjusts spindle RPM based on real-time workpiece diameter $D$:
  $$\text{RPM} = \frac{V_c}{\pi \cdot D}$$

### 6. Dynamic Tool Length & Wear Compensation (`tool_compensation.rs`)
- Supports CNC `G43` and `G44` tool offsets.
- Applies per-tool length offsets and radius wear compensations dynamically.

### 7. StallGuard Surveillance (`stall_guard.rs`)
- Real-time TMC2209/TMC2240 motor load measurement.
- Detects syringe dispenser stalls and physical obstructions with moving-average noise filtering.

---

## 🧪 Testing & Verification
```bash
cargo test -p motion
cargo test --test proptest_motion
cargo test --test determinism_test
```
