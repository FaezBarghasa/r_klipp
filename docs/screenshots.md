# 🖼️ `r_klipp` Slint GUI - Visual Gallery & Touchscreen Screenshots

This document presents the visual interface of the **`r_klipp` Touchscreen GUI (`host-ui`)**, designed for 1:1 parity with the official [KlipperScreen](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/references/KlipperScreen) layout and styling.

Targeting the **MKS PI_TS35 3.5" (480×320)** and standard **800×480** touch displays, the interface is rendered in native Rust using [Slint](https://slint.dev) with the official Material Dark color system.

---

## 📱 Interactive Touchscreen Simulator (Playwright Automation)

Below are live screen captures recorded during end-to-end Playwright touch interaction tests:

### 1. Main Jog & Motion Control Panel
Jogging along X/Y/Z axes with configurable distance increments ($0.1\text{ mm}$, $1\text{ mm}$, $10\text{ mm}$, $50\text{ mm}$, $100\text{ mm}$), per-axis homing, and `G28` Home All.

![Playwright Move Panel](img/screenshots/playwright_touch_move.png)

---

### 2. Thermal Control & Material Presets
Quick temperature target assignment for Extruder, Heated Bed, and Chamber with one-tap material presets (PLA $205^\circ / 60^\circ$, PETG $240^\circ / 80^\circ$, ABS $255^\circ / 105^\circ$, Cooldown $0^\circ / 0^\circ$).

![Playwright Temperature Panel](img/screenshots/playwright_touch_temperature.png)

---

### 3. G-Code Storage & Print File Browser
Browsing local storage and virtual SD card files with file size metadata and instant print initiation.

![Playwright Files Panel](img/screenshots/playwright_touch_files.png)

---

### 4. Active Print Job Status & Live Telemetry
Real-time visual print progress bar ($0-100\%$), current layer progression ($78/120$), remaining print duration estimation, speed/flow overrides, and Pause/Cancel buttons.

![Playwright Job Status Panel](img/screenshots/playwright_touch_job_status.png)

---

## 🎨 Complete Slint UI Component Gallery (Native `slint-viewer` Renders)

### 🖥️ Main Dashboard & System Navigation

| Main Menu (8-Tile Grid + Telemetry Pills) | App Window Top-Level Container |
| :---: | :---: |
| ![Main Menu Panel](img/screenshots/panel_main_menu.png) | ![App Window](img/screenshots/app_window_main.png) |

---

### 🎛️ Calibration & Advanced Tuning

| Bed Mesh 3D Probing Matrix | Z-Calibrate & Live Babystepping |
| :---: | :---: |
| ![Bed Mesh](img/screenshots/panel_bed_mesh.png) | ![Z-Calibrate](img/screenshots/panel_zcalibrate.png) |

| Input Shaper Resonance Calibration (ADXL345) | Bed Level Screws (Screws Tilt Calculate) |
| :---: | :---: |
| ![Input Shaper](img/screenshots/panel_input_shaper.png) | ![Bed Level](img/screenshots/panel_bed_level.png) |

| Fine Tune (Babystep, Speed, Flow) | Pressure Advance & Retraction |
| :---: | :---: |
| ![Fine Tune](img/screenshots/panel_fine_tune.png) | ![Pressure Advance](img/screenshots/panel_pressure_advance.png) |

---

### ⚙️ Hardware Control & Extruder Management

| Extrude & Retract Length/Speed Controls | Cooling Fans & PWM Controllers |
| :---: | :---: |
| ![Extrude Panel](img/screenshots/panel_extrude.png) | ![Fan Panel](img/screenshots/panel_fan.png) |

| Motion Limits (Velocity, Accel, Square Corner) | GPIO Pins & Output Switches |
| :---: | :---: |
| ![Limits Panel](img/screenshots/panel_limits.png) | ![Pins Panel](img/screenshots/panel_pins.png) |

---

### 🧵 Filament Spoolman & Peripherals

| Spoolman Filament Manager | Spool Weight & Cost Editor |
| :---: | :---: |
| ![Spoolman](img/screenshots/panel_spoolman.png) | ![Spool Editor](img/screenshots/panel_spool_editor.png) |

| LED Lighting & RGB Neopixel Controls | Live Camera Stream View |
| :---: | :---: |
| ![LED Panel](img/screenshots/panel_led.png) | ![Camera Panel](img/screenshots/panel_camera.png) |

---

### 💻 System, Terminal & Power Controls

| Interactive G-Code Console Terminal | System Load & Hardware Stats |
| :---: | :---: |
| ![Console Panel](img/screenshots/panel_console.png) | ![System Panel](img/screenshots/panel_system.png) |

| Wi-Fi & Ethernet Network Manager | Power Relays & Smart Plugs |
| :---: | :---: |
| ![Network Panel](img/screenshots/panel_network.png) | ![Power Panel](img/screenshots/panel_power.png) |

| Package & Firmware Updater | Host Shutdown & Service Restart |
| :---: | :---: |
| ![Updater Panel](img/screenshots/panel_updater.png) | ![Shutdown Panel](img/screenshots/panel_shutdown.png) |

---

## 🧪 Automated Testing Commands

To reproduce the screenshots and verify the UI on your machine:

```bash
# 1. Test and render all 29 Slint components to PNG
python3 tools/scripts/test_with_slint_viewer.py

# 2. Run the full Rust integration test suite
cargo test -p host-ui

# 3. Serve the interactive touchscreen simulator
python3 tools/scripts/serve_simulator.py
```
