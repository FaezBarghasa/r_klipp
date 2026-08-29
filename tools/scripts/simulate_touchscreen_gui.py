#!/usr/bin/env python3
"""
Touchscreen Hardware & Slint GUI Simulator / Emulation Harness for r_klipp
Simulates MKS PI_TS35 (480x320) Touchscreen Display, Touch Digitizer,
and full KlipperScreen Slint UI state transitions & Moonraker IPC.
"""

import sys
import time
import math
import json
import dataclasses
from typing import List, Dict, Tuple, Optional, Callable, Any

# --- Virtual Screen & Touch Constants ---
SCREEN_WIDTH = 480
SCREEN_HEIGHT = 320

# Colors (ANSI 24-bit TrueColor)
C_RESET = "\033[0m"
C_BOLD = "\033[1m"
C_DIM = "\033[2m"
C_CYAN = "\033[38;2;0;173;181m"      # KlipperScreen Theme.accent (#00ADB5)
C_ORANGE = "\033[38;2;255;183;77m"   # Theme.warning
C_RED = "\033[38;2;255;82;82m"       # Theme.hot (#FF5252)
C_BLUE = "\033[38;2;68;138;255m"     # Theme.cool (#448AFF)
C_GREEN = "\033[38;2;77;182;172m"    # Theme.ok (#4DB6AC)
C_BG = "\033[48;2;18;18;18m"         # Theme.bg (#121212)
C_PANEL = "\033[48;2;30;30;30m"      # Theme.panel-bg (#1E1E1E)
C_BTN = "\033[48;2;45;45;45m"        # Theme.button-bg (#2D2D2D)
C_TEXT = "\033[38;2;238;238;238m"    # Theme.text (#EEEEEE)
C_MUTED = "\033[38;2;160;160;160m"   # Theme.text-muted (#A0A0A0)

@dataclasses.dataclass
class TouchPoint:
    x: int
    y: int
    pressure: float = 1.0
    timestamp_ms: float = 0.0

@dataclasses.dataclass
class UIElement:
    id: str
    label: str
    x1: int
    y1: int
    x2: int
    y2: int
    action: str
    params: Dict[str, Any] = dataclasses.field(default_factory=dict)

    def contains(self, x: int, y: int) -> bool:
        return self.x1 <= x <= self.x2 and self.y1 <= y <= self.y2

class PrinterDigitalTwin:
    """Physics and kinematics simulation for the 3D printer hardware."""
    def __init__(self):
        self.state = "idle"  # idle, printing, paused, error, shutdown
        self.pos_x = 0.0
        self.pos_y = 0.0
        self.pos_z = 0.0
        self.pos_e = 0.0
        self.homed_x = False
        self.homed_y = False
        self.homed_z = False

        # Thermal ODE Simulation
        self.hotend_actual = 22.0
        self.hotend_target = 0.0
        self.bed_actual = 21.5
        self.bed_target = 0.0
        self.chamber_actual = 21.0
        self.chamber_target = 0.0

        # Fan PWM (0.0 - 1.0)
        self.part_fan = 0.0

        # Print Job
        self.current_file = ""
        self.print_progress = 0.0
        self.layer_current = 0
        self.layer_total = 0
        self.speed_factor = 1.0
        self.flow_factor = 1.0
        self.z_offset = 0.0

        # G-Code execution log
        self.gcode_history: List[str] = []

    def step_physics(self, dt_sec: float = 1.0):
        """Simulate heating and motion progression over time delta."""
        # Hotend heating curve (tau ~ 15s)
        if self.hotend_target > 0:
            rate = (self.hotend_target - self.hotend_actual) * (dt_sec / 12.0)
            self.hotend_actual += rate
        else:
            self.hotend_actual = max(22.0, self.hotend_actual - (dt_sec * 0.8))

        # Bed heating curve (tau ~ 40s)
        if self.bed_target > 0:
            rate = (self.bed_target - self.bed_actual) * (dt_sec / 30.0)
            self.bed_actual += rate
        else:
            self.bed_actual = max(21.5, self.bed_actual - (dt_sec * 0.3))

        # Print progress simulation if printing
        if self.state == "printing":
            self.print_progress = min(1.0, self.print_progress + (dt_sec * 0.05 * self.speed_factor))
            self.layer_current = int(self.print_progress * self.layer_total)
            if self.print_progress >= 1.0:
                self.state = "idle"

    def execute_gcode(self, cmd: str) -> str:
        cmd = cmd.strip()
        self.gcode_history.append(cmd)
        tokens = cmd.split()
        if not tokens:
            return "ok"

        main_cmd = tokens[0].upper()
        if main_cmd == "G28":
            if len(tokens) == 1:
                self.homed_x = self.homed_y = self.homed_z = True
                self.pos_x = self.pos_y = self.pos_z = 0.0
            else:
                for t in tokens[1:]:
                    axis = t.upper()
                    if "X" in axis: self.homed_x = True; self.pos_x = 0.0
                    if "Y" in axis: self.homed_y = True; self.pos_y = 0.0
                    if "Z" in axis: self.homed_z = True; self.pos_z = 0.0
            return "ok"

        elif main_cmd == "G1":
            for t in tokens[1:]:
                a = t[0].upper()
                try:
                    val = float(t[1:])
                    if a == "X": self.pos_x += val
                    elif a == "Y": self.pos_y += val
                    elif a == "Z": self.pos_z += val
                    elif a == "E": self.pos_e += val
                except ValueError:
                    pass
            return "ok"

        elif main_cmd == "M104" or main_cmd == "M109":
            for t in tokens[1:]:
                if t.upper().startswith("S"):
                    self.hotend_target = float(t[1:])
            return "ok"

        elif main_cmd == "M140" or main_cmd == "M190":
            for t in tokens[1:]:
                if t.upper().startswith("S"):
                    self.bed_target = float(t[1:])
            return "ok"

        elif main_cmd == "M106":
            for t in tokens[1:]:
                if t.upper().startswith("S"):
                    self.part_fan = float(t[1:]) / 255.0
            return "ok"

        elif main_cmd == "M112":
            self.state = "shutdown"
            self.hotend_target = 0.0
            self.bed_target = 0.0
            return "!! EMERGENCY STOP TRIGGERED !!"

        elif main_cmd == "M84":
            self.homed_x = self.homed_y = self.homed_z = False
            return "ok"

        return f"ok (executed: {cmd})"


class TouchscreenGuiEmulator:
    """Emulates the Slint 480x320 Touchscreen UI and User Interaction."""
    def __init__(self):
        self.twin = PrinterDigitalTwin()
        self.active_screen = "main_menu"
        self.screen_history: List[str] = []
        self.touch_log: List[Dict[str, Any]] = []
        self.console_lines: List[str] = [
            "r_klipp Slint UI v0.1.0 Initialized",
            "Hardware: MKS SKIPR + MKS PI_TS35 (480x320)",
            "Connected to Moonraker API at 127.0.0.1:7125",
        ]

    def get_layout(self) -> List[UIElement]:
        """Returns the active UI elements and touch hit-boxes for current screen."""
        elems = []
        # Global top header (Titlebar)
        elems.append(UIElement("hdr_home", "Home", 0, 0, 80, 40, "navigate", {"target": "main_menu"}))
        elems.append(UIElement("hdr_estop", "E-STOP", 400, 0, 480, 40, "estop", {}))

        if self.active_screen == "main_menu":
            # Heater Pills at top
            elems.append(UIElement("pill_hotend", f"Hotend: {self.twin.hotend_actual:.0f}/{self.twin.hotend_target:.0f}°", 10, 45, 155, 95, "navigate", {"target": "temperature"}))
            elems.append(UIElement("pill_bed", f"Bed: {self.twin.bed_actual:.0f}/{self.twin.bed_target:.0f}°", 165, 45, 310, 95, "navigate", {"target": "temperature"}))
            elems.append(UIElement("pill_status", f"State: {self.twin.state.upper()}", 320, 45, 470, 95, "navigate", {"target": "job_status"}))

            # 4x2 Menu Tiles
            # Row 1
            elems.append(UIElement("tile_move", "Move", 10, 105, 120, 195, "navigate", {"target": "move"}))
            elems.append(UIElement("tile_temp", "Temperature", 130, 105, 240, 195, "navigate", {"target": "temperature"}))
            elems.append(UIElement("tile_extrude", "Extrude", 250, 105, 360, 195, "navigate", {"target": "extrude"}))
            elems.append(UIElement("tile_fan", "Fan", 370, 105, 470, 195, "navigate", {"target": "fan"}))

            # Row 2
            elems.append(UIElement("tile_files", "Files", 10, 205, 120, 295, "navigate", {"target": "files"}))
            elems.append(UIElement("tile_console", "Console", 130, 205, 240, 295, "navigate", {"target": "console"}))
            elems.append(UIElement("tile_tune", "Fine Tune", 250, 205, 360, 295, "navigate", {"target": "fine_tune"}))
            elems.append(UIElement("tile_more", "System", 370, 205, 470, 295, "navigate", {"target": "system"}))

        elif self.active_screen == "move":
            # Jog buttons
            elems.append(UIElement("btn_home_all", "HOME ALL (G28)", 10, 50, 150, 95, "gcode", {"cmd": "G28"}))
            elems.append(UIElement("btn_home_xy", "HOME XY", 160, 50, 270, 95, "gcode", {"cmd": "G28 X Y"}))
            elems.append(UIElement("btn_home_z", "HOME Z", 280, 50, 370, 95, "gcode", {"cmd": "G28 Z"}))
            elems.append(UIElement("btn_disable_motors", "MOTORS OFF", 380, 50, 470, 95, "gcode", {"cmd": "M84"}))

            # XY Directional Cross
            elems.append(UIElement("btn_jog_yp", "Y +10", 180, 105, 260, 150, "gcode", {"cmd": "G1 Y10"}))
            elems.append(UIElement("btn_jog_xm", "X -10", 90, 160, 170, 205, "gcode", {"cmd": "G1 X-10"}))
            elems.append(UIElement("btn_jog_xp", "X +10", 270, 160, 350, 205, "gcode", {"cmd": "G1 X10"}))
            elems.append(UIElement("btn_jog_ym", "Y -10", 180, 215, 260, 260, "gcode", {"cmd": "G1 Y-10"}))

            # Z Jogging
            elems.append(UIElement("btn_jog_zp", "Z +5", 380, 120, 460, 175, "gcode", {"cmd": "G1 Z5"}))
            elems.append(UIElement("btn_jog_zm", "Z -5", 380, 195, 460, 250, "gcode", {"cmd": "G1 Z-5"}))

            # Back button
            elems.append(UIElement("btn_back", "◀ Back", 10, 270, 120, 310, "navigate", {"target": "main_menu"}))

        elif self.active_screen == "temperature":
            elems.append(UIElement("btn_preset_pla", "PLA (205/60)", 20, 60, 150, 120, "set_temps", {"hotend": 205.0, "bed": 60.0}))
            elems.append(UIElement("btn_preset_petg", "PETG (240/80)", 165, 60, 305, 120, "set_temps", {"hotend": 240.0, "bed": 80.0}))
            elems.append(UIElement("btn_preset_abs", "ABS (255/100)", 320, 60, 460, 120, "set_temps", {"hotend": 255.0, "bed": 100.0}))
            elems.append(UIElement("btn_cooldown", "COOLDOWN (0/0)", 20, 140, 230, 200, "set_temps", {"hotend": 0.0, "bed": 0.0}))
            elems.append(UIElement("btn_back", "◀ Back", 10, 270, 120, 310, "navigate", {"target": "main_menu"}))

        elif self.active_screen == "files":
            elems.append(UIElement("file_item_1", "🗎 voron_cube.gcode (1.2MB)", 20, 60, 460, 110, "start_print", {"filename": "voron_cube.gcode", "layers": 120}))
            elems.append(UIElement("file_item_2", "🗎 3dbenchy_pla.gcode (4.5MB)", 20, 120, 460, 170, "start_print", {"filename": "3dbenchy_pla.gcode", "layers": 240}))
            elems.append(UIElement("file_item_3", "🗎 stealthburner_body.gcode (8.1MB)", 20, 180, 460, 230, "start_print", {"filename": "stealthburner_body.gcode", "layers": 450}))
            elems.append(UIElement("btn_back", "◀ Back", 10, 270, 120, 310, "navigate", {"target": "main_menu"}))

        elif self.active_screen == "job_status":
            elems.append(UIElement("btn_pause_resume", "PAUSE/RESUME", 20, 210, 150, 260, "toggle_pause", {}))
            elems.append(UIElement("btn_cancel", "CANCEL PRINT", 165, 210, 305, 260, "cancel_print", {}))
            elems.append(UIElement("btn_tune_live", "FINE TUNE", 320, 210, 460, 260, "navigate", {"target": "fine_tune"}))
            elems.append(UIElement("btn_back", "◀ Back", 10, 270, 120, 310, "navigate", {"target": "main_menu"}))

        elif self.active_screen == "fine_tune":
            elems.append(UIElement("btn_baby_up", "Z-Offset +0.02", 20, 70, 220, 130, "babystep", {"delta": 0.02}))
            elems.append(UIElement("btn_baby_down", "Z-Offset -0.02", 240, 70, 440, 130, "babystep", {"delta": -0.02}))
            elems.append(UIElement("btn_spd_inc", "Speed +10%", 20, 145, 220, 205, "speed_factor", {"delta": 0.1}))
            elems.append(UIElement("btn_spd_dec", "Speed -10%", 240, 145, 440, 205, "speed_factor", {"delta": -0.1}))
            elems.append(UIElement("btn_back", "◀ Back", 10, 270, 120, 310, "navigate", {"target": "main_menu"}))

        return elems

    def process_touch(self, x: int, y: int, pressure: float = 1.0) -> Optional[str]:
        """Simulates touchscreen digitizer contact event and triggers UI callback."""
        touch = TouchPoint(x=x, y=y, pressure=pressure, timestamp_ms=time.time() * 1000)
        layout = self.get_layout()
        hit: Optional[UIElement] = None
        for elem in layout:
            if elem.contains(x, y):
                hit = elem
                break

        event_info = {
            "x": x,
            "y": y,
            "active_screen": self.active_screen,
            "hit_element": hit.id if hit else None,
            "action": hit.action if hit else None,
        }
        self.touch_log.append(event_info)

        if not hit:
            return None

        # Execute action
        if hit.action == "navigate":
            target = hit.params.get("target", "main_menu")
            self.screen_history.append(self.active_screen)
            self.active_screen = target
            return f"Navigated to '{target}'"

        elif hit.action == "gcode":
            cmd = hit.params.get("cmd", "")
            res = self.twin.execute_gcode(cmd)
            self.console_lines.append(f"> {cmd} ({res})")
            return f"Executed G-Code: '{cmd}' -> {res}"

        elif hit.action == "set_temps":
            h = hit.params.get("hotend", 0.0)
            b = hit.params.get("bed", 0.0)
            self.twin.hotend_target = h
            self.twin.bed_target = b
            self.console_lines.append(f"Set Target: Hotend={h:.0f}°C, Bed={b:.0f}°C")
            return f"Applied Temperature Preset: Hotend {h}°C, Bed {b}°C"

        elif hit.action == "start_print":
            fname = hit.params.get("filename", "")
            layers = hit.params.get("layers", 100)
            self.twin.current_file = fname
            self.twin.layer_total = layers
            self.twin.layer_current = 1
            self.twin.print_progress = 0.01
            self.twin.state = "printing"
            self.active_screen = "job_status"
            self.console_lines.append(f"Starting print: {fname}")
            return f"Started Print: '{fname}' ({layers} layers)"

        elif hit.action == "toggle_pause":
            if self.twin.state == "printing":
                self.twin.state = "paused"
                return "Print Paused"
            elif self.twin.state == "paused":
                self.twin.state = "printing"
                return "Print Resumed"

        elif hit.action == "cancel_print":
            self.twin.state = "idle"
            self.twin.print_progress = 0.0
            self.twin.current_file = ""
            self.active_screen = "main_menu"
            return "Print Cancelled"

        elif hit.action == "babystep":
            d = hit.params.get("delta", 0.0)
            self.twin.z_offset += d
            return f"Z-Offset babystep: {self.twin.z_offset:+.3f}mm"

        elif hit.action == "speed_factor":
            d = hit.params.get("delta", 0.0)
            self.twin.speed_factor = max(0.1, min(3.0, self.twin.speed_factor + d))
            return f"Speed factor adjusted to: {int(self.twin.speed_factor * 100)}%"

        elif hit.action == "estop":
            res = self.twin.execute_gcode("M112")
            return f"EMERGENCY STOP: {res}"

        return None

    def render_ansi_screen(self, touch_pos: Optional[Tuple[int, int]] = None) -> str:
        """Renders an accurate ASCII/ANSI representation of the 480x320 display."""
        lines = []
        width = 62  # text grid width
        lines.append(f"{C_CYAN}┌{'─' * (width - 2)}┐{C_RESET}")

        # Top Bar
        homed_str = f"{'X' if self.twin.homed_x else '-'}{'Y' if self.twin.homed_y else '-'}{'Z' if self.twin.homed_z else '-'}"
        title = f" MKS PI_TS35 [Screen: {self.active_screen.upper()}] "
        stat = f" [State: {self.twin.state.upper()}] [Homed: {homed_str}]"
        hdr = f"{title}{' ' * max(0, width - 4 - len(title) - len(stat))}{stat}"
        lines.append(f"{C_CYAN}│{C_BOLD}{C_TEXT}{hdr[:width-2]}{C_RESET}{C_CYAN}│{C_RESET}")
        lines.append(f"{C_CYAN}├{'─' * (width - 2)}┤{C_RESET}")

        # Body area
        body_lines = []
        if self.active_screen == "main_menu":
            body_lines.append(f"  {C_RED}♨ Hotend:{C_RESET} {self.twin.hotend_actual:5.1f}° / {self.twin.hotend_target:3.0f}°C   {C_BLUE}♨ Bed:{C_RESET} {self.twin.bed_actual:5.1f}° / {self.twin.bed_target:3.0f}°C")
            body_lines.append(f"  {C_GREEN}Position:{C_RESET} X:{self.twin.pos_x:6.1f}  Y:{self.twin.pos_y:6.1f}  Z:{self.twin.pos_z:6.1f} mm")
            body_lines.append("  " + "─" * 56)
            body_lines.append(f"  {C_BTN}[ 1. Move ]{C_RESET}     {C_BTN}[ 2. Temperature ]{C_RESET}  {C_BTN}[ 3. Extrude ]{C_RESET}")
            body_lines.append(f"  {C_BTN}[ 4. Fan ]{C_RESET}      {C_BTN}[ 5. Files ]{C_RESET}        {C_BTN}[ 6. Console ]{C_RESET}")
            body_lines.append(f"  {C_BTN}[ 7. Fine Tune ]{C_RESET}{C_BTN}[ 8. System ]{C_RESET}       {C_RED}[ E-STOP ]{C_RESET}")

        elif self.active_screen == "move":
            body_lines.append(f"  {C_CYAN}Move / Kinematics Control{C_RESET} (Homed: {homed_str})")
            body_lines.append(f"  Current Coord: X:{self.twin.pos_x:6.1f} Y:{self.twin.pos_y:6.1f} Z:{self.twin.pos_z:6.1f}")
            body_lines.append("  " + "─" * 56)
            body_lines.append(f"  {C_BTN}[ HOME ALL ]{C_RESET}   {C_BTN}[ Y +10 ]{C_RESET}        {C_BTN}[ Z +5 ]{C_RESET}")
            body_lines.append(f"  {C_BTN}[ X -10 ]{C_RESET}      {C_BTN}[ HOME XY ]{C_RESET}      {C_BTN}[ X +10 ]{C_RESET}")
            body_lines.append(f"  {C_BTN}[ MOTORS OFF ]{C_RESET} {C_BTN}[ Y -10 ]{C_RESET}        {C_BTN}[ Z -5 ]{C_RESET}")
            body_lines.append(f"  {C_MUTED}[ ◀ Back to Menu ]{C_RESET}")

        elif self.active_screen == "temperature":
            body_lines.append(f"  {C_RED}Hotend:{C_RESET} {self.twin.hotend_actual:5.1f} / {self.twin.hotend_target:3.0f}°C   {C_BLUE}Bed:{C_RESET} {self.twin.bed_actual:5.1f} / {self.twin.bed_target:3.0f}°C")
            body_lines.append("  " + "─" * 56)
            body_lines.append(f"  Presets: {C_BTN}[ PLA 205/60 ]{C_RESET}  {C_BTN}[ PETG 240/80 ]{C_RESET}")
            body_lines.append(f"           {C_BTN}[ ABS 255/100 ]{C_RESET} {C_BTN}[ COOLDOWN 0/0 ]{C_RESET}")
            body_lines.append(f"  {C_MUTED}[ ◀ Back to Menu ]{C_RESET}")

        elif self.active_screen == "files":
            body_lines.append(f"  {C_CYAN}G-Code Storage Browser (/gcodes){C_RESET}")
            body_lines.append("  " + "─" * 56)
            body_lines.append(f"  1. {C_GREEN}🗎 voron_cube.gcode{C_RESET}            [1.2 MB]")
            body_lines.append(f"  2. {C_GREEN}🗎 3dbenchy_pla.gcode{C_RESET}          [4.5 MB]")
            body_lines.append(f"  3. {C_GREEN}🗎 stealthburner_body.gcode{C_RESET}    [8.1 MB]")
            body_lines.append(f"  {C_MUTED}[ ◀ Back to Menu ]{C_RESET}")

        elif self.active_screen == "job_status":
            pct = int(self.twin.print_progress * 100)
            bar_len = 24
            filled = int((pct / 100.0) * bar_len)
            bar = "█" * filled + "░" * (bar_len - filled)
            body_lines.append(f"  File: {C_CYAN}{self.twin.current_file}{C_RESET}")
            body_lines.append(f"  Progress: [{bar}] {pct}%")
            body_lines.append(f"  Layer: {self.twin.layer_current} / {self.twin.layer_total} | Speed: {int(self.twin.speed_factor*100)}% | Flow: {int(self.twin.flow_factor*100)}%")
            body_lines.append(f"  Z-Offset: {self.twin.z_offset:+.3f} mm")
            body_lines.append(f"  {C_BTN}[ PAUSE/RESUME ]{C_RESET}  {C_RED}[ CANCEL PRINT ]{C_RESET}  {C_BTN}[ FINE TUNE ]{C_RESET}")
            body_lines.append(f"  {C_MUTED}[ ◀ Back to Menu ]{C_RESET}")

        elif self.active_screen == "fine_tune":
            body_lines.append(f"  {C_CYAN}Live Print Fine Tuning{C_RESET}")
            body_lines.append(f"  Babystep Z: {self.twin.z_offset:+.3f} mm  |  Speed: {int(self.twin.speed_factor*100)}%")
            body_lines.append("  " + "─" * 56)
            body_lines.append(f"  {C_BTN}[ Z +0.02 ]{C_RESET}  {C_BTN}[ Z -0.02 ]{C_RESET}  {C_BTN}[ Speed +10% ]{C_RESET} {C_BTN}[ Speed -10% ]{C_RESET}")
            body_lines.append(f"  {C_MUTED}[ ◀ Back to Menu ]{C_RESET}")

        # Pad lines to height
        while len(body_lines) < 8:
            body_lines.append("")

        for bl in body_lines:
            # Strip ANSI length calculation
            clean = bl.replace(C_RESET, "").replace(C_BOLD, "").replace(C_DIM, "").replace(C_CYAN, "").replace(C_ORANGE, "").replace(C_RED, "").replace(C_BLUE, "").replace(C_GREEN, "").replace(C_BG, "").replace(C_PANEL, "").replace(C_BTN, "").replace(C_TEXT, "").replace(C_MUTED, "")
            pad = max(0, width - 4 - len(clean))
            lines.append(f"{C_CYAN}│{C_RESET} {bl}{' ' * pad} {C_CYAN}│{C_RESET}")

        if touch_pos:
            tx, ty = touch_pos
            touch_indicator = f" {C_ORANGE}◉ TOUCH DIGITIZER: ({tx:3d}, {ty:3d}){C_RESET} "
            clean_ti = f" ◉ TOUCH DIGITIZER: ({tx:3d}, {ty:3d}) "
            pad = max(0, width - 4 - len(clean_ti))
            lines.append(f"{C_CYAN}├{'─' * (width - 2)}┤{C_RESET}")
            lines.append(f"{C_CYAN}│{C_RESET}{touch_indicator}{' ' * pad}{C_CYAN}│{C_RESET}")

        lines.append(f"{C_CYAN}└{'─' * (width - 2)}┘{C_RESET}")
        return "\n".join(lines)


def run_full_touchscreen_emulation_suite() -> Dict[str, Any]:
    """Runs automated user touch interaction scenarios across the Slint GUI."""
    emu = TouchscreenGuiEmulator()
    results = []

    print(f"\n{C_BOLD}{C_CYAN}==============================================================={C_RESET}")
    print(f"{C_BOLD}{C_CYAN}  MKS PI_TS35 Touchscreen & Slint GUI Hardware Emulation Harness{C_RESET}")
    print(f"{C_BOLD}{C_CYAN}==============================================================={C_RESET}\n")

    # Step 1: Initial Render
    print(f"{C_BOLD}▶ Step 1: Initial Boot to Main Menu{C_RESET}")
    print(emu.render_ansi_screen())
    results.append({"step": 1, "screen": emu.active_screen, "action": "boot", "status": "PASS"})
    time.sleep(0.1)

    # Step 2: User taps "Move" tile (x=60, y=150)
    print(f"\n{C_BOLD}▶ Step 2: User Taps 'Move' Tile (X=60, Y=150){C_RESET}")
    act = emu.process_touch(60, 150)
    print(emu.render_ansi_screen(touch_pos=(60, 150)))
    print(f"  └─ Action: {C_GREEN}{act}{C_RESET}")
    assert emu.active_screen == "move", "Failed to navigate to move panel"
    results.append({"step": 2, "screen": emu.active_screen, "action": act, "status": "PASS"})
    time.sleep(0.1)

    # Step 3: User taps "HOME ALL (G28)" (x=80, y=70)
    print(f"\n{C_BOLD}▶ Step 3: User Taps 'HOME ALL (G28)' (X=80, Y=70){C_RESET}")
    act = emu.process_touch(80, 70)
    print(emu.render_ansi_screen(touch_pos=(80, 70)))
    print(f"  └─ Action: {C_GREEN}{act}{C_RESET}")
    assert emu.twin.homed_x and emu.twin.homed_y and emu.twin.homed_z, "Homing failed"
    results.append({"step": 3, "screen": emu.active_screen, "action": act, "status": "PASS"})
    time.sleep(0.1)

    # Step 4: User jogs Z +5 (x=420, y=150) twice
    print(f"\n{C_BOLD}▶ Step 4: User Taps 'Z +5' (X=420, Y=150) twice{C_RESET}")
    emu.process_touch(420, 150)
    act = emu.process_touch(420, 150)
    print(emu.render_ansi_screen(touch_pos=(420, 150)))
    print(f"  └─ Action: {C_GREEN}{act}{C_RESET}")
    assert emu.twin.pos_z == 10.0, f"Expected Z=10.0, got {emu.twin.pos_z}"
    results.append({"step": 4, "screen": emu.active_screen, "action": act, "status": "PASS"})
    time.sleep(0.1)

    # Step 5: User taps "Back" (x=50, y=290)
    print(f"\n{C_BOLD}▶ Step 5: User Taps 'Back' to Main Menu (X=50, Y=290){C_RESET}")
    act = emu.process_touch(50, 290)
    print(emu.render_ansi_screen(touch_pos=(50, 290)))
    print(f"  └─ Action: {C_GREEN}{act}{C_RESET}")
    assert emu.active_screen == "main_menu"
    results.append({"step": 5, "screen": emu.active_screen, "action": act, "status": "PASS"})
    time.sleep(0.1)

    # Step 6: User taps "Temperature" tile (x=180, y=150)
    print(f"\n{C_BOLD}▶ Step 6: User Taps 'Temperature' (X=180, Y=150){C_RESET}")
    act = emu.process_touch(180, 150)
    print(emu.render_ansi_screen(touch_pos=(180, 150)))
    print(f"  └─ Action: {C_GREEN}{act}{C_RESET}")
    assert emu.active_screen == "temperature"
    results.append({"step": 6, "screen": emu.active_screen, "action": act, "status": "PASS"})
    time.sleep(0.1)

    # Step 7: User selects PLA Preset (205°C / 60°C) (x=80, y=90) and heats up for physics steps
    print(f"\n{C_BOLD}▶ Step 7: User Taps 'PLA Preset (205/60)' (X=80, Y=90) & Physics Step{C_RESET}")
    act = emu.process_touch(80, 90)
    for _ in range(16):
        emu.twin.step_physics(dt_sec=2.0)
    print(emu.render_ansi_screen(touch_pos=(80, 90)))
    print(f"  └─ Action: {C_GREEN}{act}{C_RESET}")
    assert emu.twin.hotend_actual > 150.0 and emu.twin.bed_actual > 45.0, "Heating ODE failed"
    results.append({"step": 7, "screen": emu.active_screen, "action": act, "status": "PASS"})
    time.sleep(0.1)

    # Step 8: User taps "Back" and navigates to "Files" (x=50, y=250)
    print(f"\n{C_BOLD}▶ Step 8: User Taps 'Back' then 'Files' (X=50, Y=250){C_RESET}")
    emu.process_touch(50, 290)  # Back
    act = emu.process_touch(50, 250)  # Files
    print(emu.render_ansi_screen(touch_pos=(50, 250)))
    print(f"  └─ Action: {C_GREEN}{act}{C_RESET}")
    assert emu.active_screen == "files"
    results.append({"step": 8, "screen": emu.active_screen, "action": act, "status": "PASS"})
    time.sleep(0.1)

    # Step 9: User taps "voron_cube.gcode" to start printing (x=200, y=85)
    print(f"\n{C_BOLD}▶ Step 9: User Starts Print 'voron_cube.gcode' (X=200, Y=85){C_RESET}")
    act = emu.process_touch(200, 85)
    for _ in range(6):
        emu.twin.step_physics(dt_sec=1.5)
    print(emu.render_ansi_screen(touch_pos=(200, 85)))
    print(f"  └─ Action: {C_GREEN}{act}{C_RESET}")
    assert emu.twin.state == "printing" and emu.twin.print_progress > 0.0, "Print start failed"
    results.append({"step": 9, "screen": emu.active_screen, "action": act, "status": "PASS"})
    time.sleep(0.1)

    # Step 10: User taps "FINE TUNE" (x=380, y=230) -> Babysteps Z -0.02mm
    print(f"\n{C_BOLD}▶ Step 10: User Taps 'FINE TUNE' (X=380, Y=230) -> Babystep Z -0.02mm{C_RESET}")
    emu.process_touch(380, 230)
    act = emu.process_touch(300, 100) # Babystep down
    print(emu.render_ansi_screen(touch_pos=(300, 100)))
    print(f"  └─ Action: {C_GREEN}{act}{C_RESET}")
    assert emu.twin.z_offset == -0.02, "Babystep failed"
    results.append({"step": 10, "screen": emu.active_screen, "action": act, "status": "PASS"})
    time.sleep(0.1)

    # Step 11: Emergency Stop Test (x=440, y=20)
    print(f"\n{C_BOLD}▶ Step 11: User Taps Top-Right 'E-STOP' (X=440, Y=20){C_RESET}")
    act = emu.process_touch(440, 20)
    print(emu.render_ansi_screen(touch_pos=(440, 20)))
    print(f"  └─ Action: {C_RED}{act}{C_RESET}")
    assert emu.twin.state == "shutdown", "E-Stop state transition failed"
    results.append({"step": 11, "screen": emu.active_screen, "action": act, "status": "PASS"})

    print(f"\n{C_BOLD}{C_GREEN}✔ All 11 Touchscreen Interaction Scenarios Passed Successfully!{C_RESET}\n")
    return {
        "total_steps": 11,
        "passed": 11,
        "failed": 0,
        "scenarios": results,
    }

if __name__ == "__main__":
    res = run_full_touchscreen_emulation_suite()
    sys.exit(0 if res["failed"] == 0 else 1)
