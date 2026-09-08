# `host-ui`: Slint 1.x Touchscreen Operator Interface

`host-ui` is a modern, responsive local operator touch screen interface built with [Slint](https://slint.dev). Designed for single-board computers (Raspberry Pi, MKS TS35), it provides zero-latency machine status visualization and manual jog control.

---

## ✨ Features & Interface Panels

- **Live Dashboard**: Visualizes nozzle/bed temperatures, live X/Y/Z coordinates, print progress percentage, and elapsed/remaining time.
- **Manual Jog Controls**: Directional jog buttons with configurable step increments ($0.1\text{ mm}$, $1.0\text{ mm}$, $10.0\text{ mm}$, $50.0\text{ mm}$) and per-axis homing (`Home All`, `Home X/Y`, `Home Z`).
- **G-Code File Browser**: Lists local and uploaded print files with one-touch start/pause/cancel controls.
- **Interactive Terminal Console**: Real-time G-Code command execution and response logging.
- **Emergency Stop Button**: Direct instant-response E-Stop trigger.

---

## 🎨 UI Definition & Visual Gallery

The graphical interface is defined declaratively in [`ui/main.slint`](ui/main.slint) and compiles directly into native Rust GUI types via `slint-build`.

See the complete [🖼️ Visual Screenshots Gallery](../docs/screenshots.md) for all 29 panels and Playwright touch interactions.

### Interface Previews

| Main Menu (8-Tile Grid) | Jog & Motion Control (X/Y/Z) |
| :---: | :---: |
| ![Main Menu](../docs/img/screenshots/panel_main_menu.png) | ![Move Panel](../docs/img/screenshots/playwright_touch_move.png) |

| Thermal Presets & Heating | Live Print Job Status |
| :---: | :---: |
| ![Temperature Panel](../docs/img/screenshots/playwright_touch_temperature.png) | ![Job Status Panel](../docs/img/screenshots/playwright_touch_job_status.png) |

| Bed Mesh 3D Matrix | Z-Calibrate & Babystepping |
| :---: | :---: |
| ![Bed Mesh](../docs/img/screenshots/panel_bed_mesh.png) | ![Z-Calibrate](../docs/img/screenshots/panel_zcalibrate.png) |
