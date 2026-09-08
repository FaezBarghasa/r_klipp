#!/usr/bin/env python3
"""
Test Slint UI Panels and AppWindow with slint-viewer
Validates syntax, loads components, and captures PNG screenshots for all panels.
"""

import subprocess
import os
import sys

BASE_DIR = "/home/jrad/RustroverProjects/r_klipp-workspace/r_klipp"
UI_DIR = os.path.join(BASE_DIR, "host-ui", "ui")
ARTIFACT_DIR = "/home/jrad/.gemini/antigravity-ide/brain/541af069-311c-4f5b-a976-162c43a84133"
SNAPSHOT_DIR = os.path.join(ARTIFACT_DIR, "slint_viewer_snapshots")

os.makedirs(SNAPSHOT_DIR, exist_ok=True)

PANELS = [
    ("main_menu", "panels/main_menu.slint", "MainMenuPanel"),
    ("move", "panels/move.slint", "MovePanel"),
    ("temperature", "panels/temperature.slint", "TemperaturePanel"),
    ("job_status", "panels/job_status.slint", "JobStatusPanel"),
    ("extrude", "panels/extrude.slint", "ExtrudePanel"),
    ("fan", "panels/fan.slint", "FanPanel"),
    ("bed_level", "panels/bed_level.slint", "BedLevelPanel"),
    ("bed_mesh", "panels/bed_mesh.slint", "BedMeshPanel"),
    ("zcalibrate", "panels/zcalibrate.slint", "ZCalibratePanel"),
    ("input_shaper", "panels/input_shaper.slint", "InputShaperPanel"),
    ("limits", "panels/limits.slint", "LimitsPanel"),
    ("pressure_advance", "panels/pressure_advance.slint", "PressureAdvancePanel"),
    ("retraction", "panels/retraction.slint", "RetractionPanel"),
    ("fine_tune", "panels/fine_tune.slint", "FineTunePanel"),
    ("pins", "panels/pins.slint", "PinsPanel"),
    ("network", "panels/network.slint", "NetworkPanel"),
    ("system", "panels/system.slint", "SystemPanel"),
    ("settings", "panels/settings.slint", "SettingsPanel"),
    ("updater", "panels/updater.slint", "UpdaterPanel"),
    ("power", "panels/power.slint", "PowerPanel"),
    ("shutdown", "panels/shutdown.slint", "ShutdownPanel"),
    ("camera", "panels/camera.slint", "CameraPanel"),
    ("spoolman", "panels/spoolman.slint", "SpoolmanPanel"),
    ("spool_editor", "panels/spool_editor.slint", "SpoolEditorPanel"),
    ("gcode_macros", "panels/gcode_macros.slint", "GCodeMacrosPanel"),
    ("exclude", "panels/exclude.slint", "ExcludeObjectPanel"),
    ("led", "panels/led.slint", "LedPanel"),
    ("files", "panels/files.slint", "FilesPanel"),
    ("console", "panels/console.slint", "ConsolePanel"),
]

def check_main():
    print(">>> 1. Checking main.slint with slint-viewer --check...")
    cmd = ["slint-viewer", "-I", UI_DIR, "--check", os.path.join(UI_DIR, "main.slint")]
    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0:
        print(f"FAILED main.slint check:\n{res.stderr}")
        return False
    print("✔ main.slint compiled with 0 errors/warnings.")
    return True

def capture_app_window():
    print("\n>>> 2. Capturing AppWindow with slint-viewer...")
    out_png = os.path.join(SNAPSHOT_DIR, "app_window_main.png")
    cmd = [
        "slint-viewer",
        "-I", UI_DIR,
        "--screenshot", out_png,
        os.path.join(UI_DIR, "main.slint")
    ]
    res = subprocess.run(cmd, capture_output=True, text=True)
    if res.returncode != 0 or not os.path.exists(out_png):
        print(f"FAILED AppWindow screenshot:\n{res.stderr}")
        return False
    size = os.path.getsize(out_png)
    print(f"✔ AppWindow screenshot captured: {out_png} ({size} bytes)")
    return True

def capture_all_panels():
    print("\n>>> 3. Testing and capturing all individual panels...")
    success_count = 0
    for name, rel_path, comp_name in PANELS:
        full_path = os.path.join(UI_DIR, rel_path)
        out_png = os.path.join(SNAPSHOT_DIR, f"panel_{name}.png")
        cmd = [
            "slint-viewer",
            "-I", UI_DIR,
            "--component", comp_name,
            "--screenshot", out_png,
            full_path
        ]
        res = subprocess.run(cmd, capture_output=True, text=True)
        if res.returncode == 0 and os.path.exists(out_png) and os.path.getsize(out_png) > 500:
            size = os.path.getsize(out_png)
            print(f"  ✔ [{comp_name}] captured -> panel_{name}.png ({size} bytes)")
            success_count += 1
        else:
            print(f"  ❌ [{comp_name}] failed: {res.stderr.strip() or res.stdout.strip()}")
            
    print(f"\nCompleted panel captures: {success_count}/{len(PANELS)} successful.")
    return success_count == len(PANELS)

if __name__ == "__main__":
    c1 = check_main()
    c2 = capture_app_window()
    c3 = capture_all_panels()
    if c1 and c2 and c3:
        print("\n🎉 ALL SLINT-VIEWER TESTS PASSED SUCCESSFULLY!")
        sys.exit(0)
    else:
        print("\n❌ SOME SLINT-VIEWER TESTS FAILED.")
        sys.exit(1)
