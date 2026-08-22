# `r_klipp_sim`: Deterministic Simulator & Trajectory Visualizer

`r_klipp_sim` is a host-side simulation environment providing virtual microcontrollers (`FakeMcu`), mock hardware abstraction layers, and SVG/CSV trajectory visualization pipelines for offline motion testing.

---

## 💻 Features & Capabilities

### 1. `FakeMcu` & Protocol Emulator
- Emulates the full serial handshake, DPLL clock synchronizer, and command dispatch loop of a physical MCU on Linux, macOS, and Windows.

### 2. End-to-End Simulation Pipeline
- Feeds streaming G-Code blocks through the lexer, kinematics engine, lookahead queue, and step generator, recording step intervals and dynamic states with clock-tick fidelity.

### 3. Trajectory Export & Visualization (`export.rs`)
- **CSV Data (`target/sim_trajectory.csv`)**: High-resolution kinematic data including time $t$, coordinates $(X,Y,Z,E)$, velocity $v(t)$, acceleration $a(t)$, and jerk $j(t)$.
- **Vector SVG (`target/sim_toolpath.svg`)**: Visualizes the geometric toolpath with color-coded speed gradients for cornering inspection.

---

## 🧪 Running the Simulator
```bash
# Run simulator pipeline and export visualizations
cargo test -p sim test_gcode_pipeline_with_export -- --nocapture
```
