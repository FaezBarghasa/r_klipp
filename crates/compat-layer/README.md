# `r_klipp_compat_layer`: Multi-Target Machine Profiles & Configuration Engine

`compat-layer` handles legacy `printer.cfg` parsing, schema migrations, and strict validation of multi-target machine profiles across **3D Printers**, **Pick & Place (PnP)** machines, and **3–5 Axis CNC Mills**.

---

## ⚙️ Target Profiles & Validation

Defined in [`crates/compat-layer/src/profiles.rs`](src/profiles.rs):

### 1. `MachineType::ThreeDPrinter`
- Validates hotends, heated beds, thermistor types, and pressure advance parameters.
- Reference template: [`profiles/3d_printer.toml`](../../profiles/3d_printer.toml)

### 2. `MachineType::PnpPip`
- Validates dual-head kinematics ($Z_1, Z_2, C_1, C_2$), vacuum valves, feeder communication channels, and StallGuard torque thresholds.
- Reference template: [`profiles/pnp.toml`](../../profiles/pnp.toml)

### 3. `MachineType::Cnc`
- Validates spindle RPM bounds, VFD PWM pins, RTCP kinematics parameters, touch probe inputs, and enclosure door safety interlocks.
- Reference template: [`profiles/cnc.toml`](../../profiles/cnc.toml)

---

## 🧪 Testing
```bash
cargo test -p compat-layer
```
