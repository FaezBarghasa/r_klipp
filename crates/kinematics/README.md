# `r_klipp_kinematics`: Universal Kinematics Engine

`r_klipp_kinematics` provides forward and inverse kinematics transformations across additive, electronics assembly, and subtractive machine configurations.

---

## 🧮 Supported Kinematics Models

All models implement the universal `Kinematics` trait:

```rust
pub trait Kinematics {
    fn cartesian_to_motors(&self, cartesian: [f64; 4]) -> Result<[f64; 4], KinematicsError>;
    fn motors_to_cartesian(&self, motors: [f64; 4]) -> Result<[f64; 4], KinematicsError>;
}
```

### 1. Cartesian & CoreXY
- **`CartesianKinematics`**: Direct 1:1 mapping of $X, Y, Z, E$.
- **`CoreXYKinematics`**: Dual-belt $A = X + Y, B = X - Y$ coupling.

### 2. Delta
- **`DeltaKinematics`**: Solves 3-tower rod trigonometry with radius and diagonal rod geometry constraints.

### 3. Dual-Head Pick & Place (PnP / PIP)
- **`DualHeadPnpKinematics`** ([`src/pnp_dual_head.rs`](src/pnp_dual_head.rs)):
  - Dual independent vertical Z-axes ($Z_1, Z_2$) and rotary theta C-axes ($C_1, C_2$).
  - Enforces dynamic toolhead collision envelopes and nozzle clearance boundaries.

### 4. 5-Axis RTCP (CNC Milling)
- **`FiveAxisRtcpKinematics`** ([`src/rtcp.rs`](src/rtcp.rs)):
  - 5-Axis Table-Table AC Trunnion kinematics.
  - Implements **Rotary Tool Center Point (RTCP)** preservation: rotates the part coordinate frame around $A$ and $C$ rotary axes while automatically compensating $X, Y, Z$ positions so the tool tip stays locked to the programmed workpiece path.

---

## 🧪 Testing
```bash
cargo test -p kinematics
```
