#!/usr/bin/env python3
"""
Interactive HTML5 Visual Test & Touchscreen Emulator for Slint GUI (r_klipp)
Renders 1:1 pixel-accurate MKS PI_TS35 (480x320) display with live touch interactions.
"""

import sys
import os

HTML_CONTENT = """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>MKS PI_TS35 Slint GUI Touchscreen Simulator</title>
<link href="https://fonts.googleapis.com/css2?family=Roboto:wght@400;500;700&family=JetBrains+Mono:wght@400;600&display=swap" rel="stylesheet">
<style>
  :root {
    --bg-dark: #121212;
    --panel-bg: #1E1E1E;
    --bar-bg: #1A1A1A;
    --accent: #00ADB5;
    --accent-hover: #00D9E6;
    --button-bg: #2D2D2D;
    --button-border: #404040;
    --text: #EEEEEE;
    --text-muted: #A0A0A0;
    --hot: #FF5252;
    --cool: #448AFF;
    --ok: #4DB6AC;
    --warning: #FFB74D;
    --error: #CF6679;
  }

  * { box-sizing: border-box; margin: 0; padding: 0; user-select: none; }
  body {
    background: #0a0a0c;
    color: #e0e0e0;
    font-family: 'Roboto', sans-serif;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 20px;
  }

  .header-title {
    text-align: center;
    margin-bottom: 20px;
  }
  .header-title h1 {
    font-size: 22px;
    color: var(--accent);
    letter-spacing: 1px;
  }
  .header-title p {
    font-size: 13px;
    color: var(--text-muted);
  }

  /* Hardware Bezel (MKS TS35 Frame) */
  .hardware-case {
    background: linear-gradient(145deg, #2a2b30, #1c1d22);
    border: 3px solid #3c3e47;
    border-radius: 16px;
    padding: 24px 20px 20px 20px;
    box-shadow: 0 20px 50px rgba(0,0,0,0.8), inset 0 1px 2px rgba(255,255,255,0.1);
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .case-header {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 6px 12px 6px;
    font-size: 11px;
    font-weight: 700;
    color: #787a85;
    letter-spacing: 1.5px;
    text-transform: uppercase;
  }

  /* Touchscreen Display Area (480x320 exact) */
  .touch-display {
    width: 480px;
    height: 320px;
    background: var(--bg-dark);
    border: 2px solid #000;
    border-radius: 4px;
    overflow: hidden;
    position: relative;
    box-shadow: inset 0 0 10px rgba(0,0,0,0.9);
    display: flex;
    flex-direction: column;
  }

  /* Titlebar */
  .titlebar {
    height: 40px;
    background: var(--bar-bg);
    border-bottom: 1px solid #282828;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 10px;
  }
  .titlebar-left, .titlebar-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .btn-header {
    background: var(--button-bg);
    border: 1px solid var(--button-border);
    color: var(--text);
    border-radius: 4px;
    padding: 4px 10px;
    font-size: 12px;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.1s;
  }
  .btn-header:active { background: var(--accent); color: #000; }
  .btn-estop {
    background: #c62828;
    border-color: #e53935;
    color: #fff;
  }
  .btn-estop:active { background: #ff1744; }

  .status-badge {
    font-size: 11px;
    padding: 2px 6px;
    border-radius: 3px;
    background: #2a2a2a;
    color: var(--ok);
    font-family: 'JetBrains Mono', monospace;
  }

  /* Screen Content Viewport */
  .screen-viewport {
    flex: 1;
    overflow-y: auto;
    position: relative;
    background: var(--bg-dark);
  }

  .panel-screen {
    display: none;
    height: 100%;
    padding: 10px;
    flex-direction: column;
    gap: 8px;
  }
  .panel-screen.active {
    display: flex;
  }

  /* Main Menu Grid */
  .heater-pills {
    display: grid;
    grid-template-columns: 1fr 1fr 1fr;
    gap: 8px;
    height: 52px;
  }
  .temp-pill {
    background: var(--panel-bg);
    border: 1px solid #333;
    border-radius: 6px;
    padding: 6px 8px;
    display: flex;
    flex-direction: column;
    justify-content: center;
    cursor: pointer;
  }
  .temp-pill:active { border-color: var(--accent); background: #252525; }
  .temp-pill-label { font-size: 10px; color: var(--text-muted); }
  .temp-pill-val { font-size: 14px; font-weight: 700; color: var(--accent); }

  .menu-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    grid-template-rows: repeat(2, 1fr);
    gap: 8px;
    flex: 1;
  }
  .menu-tile {
    background: var(--button-bg);
    border: 1px solid var(--button-border);
    border-radius: 6px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    cursor: pointer;
    font-size: 12px;
    font-weight: 600;
    color: var(--text);
    transition: transform 0.05s, background 0.1s;
  }
  .menu-tile:active {
    transform: scale(0.96);
    background: var(--accent);
    color: #000;
  }
  .menu-tile-icon { font-size: 20px; }

  /* Move Panel */
  .move-container {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 10px;
    height: 100%;
  }
  .jog-pad {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    grid-template-rows: repeat(3, 1fr);
    gap: 6px;
  }
  .btn-jog {
    background: var(--button-bg);
    border: 1px solid var(--button-border);
    border-radius: 6px;
    color: var(--text);
    font-weight: 700;
    font-size: 13px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }
  .btn-jog:active { background: var(--cool); color: #fff; }

  /* Temperature Presets */
  .preset-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 8px;
  }
  .preset-card {
    background: var(--panel-bg);
    border: 1px solid var(--button-border);
    border-radius: 6px;
    padding: 12px;
    text-align: center;
    cursor: pointer;
  }
  .preset-card:active { border-color: var(--accent); background: #2a2a2a; }

  /* Job Status Progress */
  .progress-card {
    background: var(--panel-bg);
    border-radius: 8px;
    padding: 12px;
    display: flex;
    flex-direction: column;
    gap: 8px;
  }
  .progress-bar-bg {
    height: 14px;
    background: #2b2b2b;
    border-radius: 7px;
    overflow: hidden;
  }
  .progress-bar-fill {
    height: 100%;
    width: 65%;
    background: linear-gradient(90deg, var(--accent), #00e5ff);
    transition: width 0.3s;
  }

  /* Touch Ripple Indicator */
  .touch-ripple {
    position: absolute;
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: rgba(0, 173, 181, 0.4);
    border: 2px solid var(--accent);
    transform: translate(-50%, -50%) scale(0);
    animation: ripple 0.4s ease-out forwards;
    pointer-events: none;
    z-index: 999;
  }
  @keyframes ripple {
    0% { transform: translate(-50%, -50%) scale(0.2); opacity: 1; }
    100% { transform: translate(-50%, -50%) scale(1.5); opacity: 0; }
  }

  /* Diagnostic Footer */
  .diag-panel {
    margin-top: 15px;
    width: 480px;
    background: #15161a;
    border: 1px solid #2d2f38;
    border-radius: 8px;
    padding: 10px 14px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px;
    color: #9aa0a6;
    display: flex;
    justify-content: space-between;
  }
</style>
</head>
<body>

<div class="header-title">
  <h1>r_klipp Slint GUI Touchscreen Simulator</h1>
  <p>1:1 Hardware Pixel Emulation of MKS PI_TS35 (480x320) & KlipperScreen Parity</p>
</div>

<div class="hardware-case">
  <div class="case-header">
    <span>MKS SKIPR HOST</span>
    <span>TS35 DISPLAY (480×320)</span>
  </div>

  <div class="touch-display" id="touchDisplay" onclick="createTouchRipple(event)">
    <!-- Top Titlebar -->
    <div class="titlebar">
      <div class="titlebar-left">
        <button class="btn-header" onclick="navTo('main_menu')">⌂ Home</button>
        <span class="status-badge" id="screenTitle">MAIN_MENU</span>
      </div>
      <div class="titlebar-right">
        <span class="status-badge" style="color: var(--warning);" id="stateBadge">IDLE</span>
        <button class="btn-header btn-estop" onclick="triggerEstop()">E-STOP</button>
      </div>
    </div>

    <!-- Viewport Screens -->
    <div class="screen-viewport">
      
      <!-- 1. MAIN MENU SCREEN -->
      <div class="panel-screen active" id="screen_main_menu">
        <div class="heater-pills">
          <div class="temp-pill" onclick="navTo('temperature')">
            <span class="temp-pill-label">Hotend</span>
            <span class="temp-pill-val" id="pillHotend">215.5° / 220°</span>
          </div>
          <div class="temp-pill" onclick="navTo('temperature')">
            <span class="temp-pill-label">Heater Bed</span>
            <span class="temp-pill-val" id="pillBed">60.2° / 65°</span>
          </div>
          <div class="temp-pill" onclick="navTo('job_status')">
            <span class="temp-pill-label">Print State</span>
            <span class="temp-pill-val" style="color: var(--ok);">STANDBY</span>
          </div>
        </div>

        <div class="menu-grid">
          <div class="menu-tile" onclick="navTo('move')">
            <span class="menu-tile-icon">✥</span>
            <span>Move</span>
          </div>
          <div class="menu-tile" onclick="navTo('temperature')">
            <span class="menu-tile-icon">♨</span>
            <span>Temperature</span>
          </div>
          <div class="menu-tile" onclick="navTo('extrude')">
            <span class="menu-tile-icon">⤓</span>
            <span>Extrude</span>
          </div>
          <div class="menu-tile" onclick="navTo('fan')">
            <span class="menu-tile-icon">🌀</span>
            <span>Fan</span>
          </div>
          <div class="menu-tile" onclick="navTo('files')">
            <span class="menu-tile-icon">📁</span>
            <span>Files</span>
          </div>
          <div class="menu-tile" onclick="navTo('job_status')">
            <span class="menu-tile-icon">⏱</span>
            <span>Job Status</span>
          </div>
          <div class="menu-tile" onclick="navTo('fine_tune')">
            <span class="menu-tile-icon">⚙</span>
            <span>Fine Tune</span>
          </div>
          <div class="menu-tile" onclick="navTo('system')">
            <span class="menu-tile-icon">ℹ</span>
            <span>System</span>
          </div>
        </div>
      </div>

      <!-- 2. MOVE SCREEN -->
      <div class="panel-screen" id="screen_move">
        <div style="display: flex; justify-content: space-between; align-items: center;">
          <span style="font-size: 13px; font-weight: 700; color: var(--accent);">Move Toolhead (Homed: XYZ)</span>
          <button class="btn-header" onclick="navTo('main_menu')">◀ Back</button>
        </div>
        <div class="move-container">
          <div class="jog-pad">
            <div></div>
            <div class="btn-jog" onclick="logGcode('G1 Y+10')">Y+</div>
            <div></div>
            <div class="btn-jog" onclick="logGcode('G1 X-10')">X-</div>
            <div class="btn-jog" style="background: var(--panel-bg); color: var(--accent);" onclick="logGcode('G28')">G28</div>
            <div class="btn-jog" onclick="logGcode('G1 X+10')">X+</div>
            <div></div>
            <div class="btn-jog" onclick="logGcode('G1 Y-10')">Y-</div>
            <div></div>
          </div>
          <div style="display: flex; flex-direction: column; gap: 8px;">
            <div style="display: flex; gap: 8px;">
              <div class="btn-jog" style="flex: 1; height: 48px;" onclick="logGcode('G1 Z+5')">Z+5</div>
              <div class="btn-jog" style="flex: 1; height: 48px;" onclick="logGcode('G1 Z-5')">Z-5</div>
            </div>
            <button class="btn-header" style="height: 38px;" onclick="logGcode('M84')">Disable Steppers (M84)</button>
            <div style="font-size: 11px; color: var(--text-muted); font-family: monospace;">
              Coord: X:125.4 Y:100.0 Z:14.8
            </div>
          </div>
        </div>
      </div>

      <!-- 3. TEMPERATURE SCREEN -->
      <div class="panel-screen" id="screen_temperature">
        <div style="display: flex; justify-content: space-between; align-items: center;">
          <span style="font-size: 13px; font-weight: 700; color: var(--hot);">Thermal Controls & Presets</span>
          <button class="btn-header" onclick="navTo('main_menu')">◀ Back</button>
        </div>
        <div class="preset-grid">
          <div class="preset-card" onclick="setTemps(205, 60)">
            <div style="font-weight: 700; color: var(--text);">PLA</div>
            <div style="font-size: 12px; color: var(--accent);">205° / 60°</div>
          </div>
          <div class="preset-card" onclick="setTemps(240, 80)">
            <div style="font-weight: 700; color: var(--text);">PETG</div>
            <div style="font-size: 12px; color: var(--accent);">240° / 80°</div>
          </div>
          <div class="preset-card" onclick="setTemps(255, 100)">
            <div style="font-weight: 700; color: var(--text);">ABS</div>
            <div style="font-size: 12px; color: var(--accent);">255° / 100°</div>
          </div>
          <div class="preset-card" onclick="setTemps(0, 0)" style="grid-column: span 3; border-color: var(--cool);">
            <div style="font-weight: 700; color: var(--cool);">COOLDOWN ALL (0°C)</div>
          </div>
        </div>
      </div>

      <!-- 4. JOB STATUS SCREEN -->
      <div class="panel-screen" id="screen_job_status">
        <div style="display: flex; justify-content: space-between; align-items: center;">
          <span style="font-size: 13px; font-weight: 700; color: var(--accent);">voron_cube_v2.gcode</span>
          <button class="btn-header" onclick="navTo('main_menu')">◀ Back</button>
        </div>
        <div class="progress-card">
          <div style="display: flex; justify-content: space-between; font-size: 12px;">
            <span>Print Progress: 64%</span>
            <span>Layer 78 / 120</span>
          </div>
          <div class="progress-bar-bg">
            <div class="progress-bar-fill"></div>
          </div>
          <div style="display: flex; justify-content: space-between; font-size: 11px; color: var(--text-muted);">
            <span>Speed: 105%</span>
            <span>Time Left: ~18m</span>
            <span>Flow: 100%</span>
          </div>
        </div>
        <div style="display: flex; gap: 8px; margin-top: 4px;">
          <button class="btn-header" style="flex: 1; height: 38px;" onclick="alert('Print Paused')">Pause</button>
          <button class="btn-header btn-estop" style="flex: 1; height: 38px;" onclick="navTo('main_menu')">Cancel Print</button>
        </div>
      </div>

      <!-- 5. FILES SCREEN -->
      <div class="panel-screen" id="screen_files">
        <div style="display: flex; justify-content: space-between; align-items: center;">
          <span style="font-size: 13px; font-weight: 700; color: var(--accent);">G-Code Files (/gcodes)</span>
          <button class="btn-header" onclick="navTo('main_menu')">◀ Back</button>
        </div>
        <div style="display: flex; flex-direction: column; gap: 6px;">
          <div class="preset-card" style="display: flex; justify-content: space-between; padding: 8px;" onclick="startPrint('voron_cube_v2.gcode')">
            <span>🗎 voron_cube_v2.gcode</span>
            <span style="color: var(--text-muted);">1.2 MB</span>
          </div>
          <div class="preset-card" style="display: flex; justify-content: space-between; padding: 8px;" onclick="startPrint('3dbenchy_pla.gcode')">
            <span>🗎 3dbenchy_pla.gcode</span>
            <span style="color: var(--text-muted);">4.5 MB</span>
          </div>
        </div>
      </div>

      <!-- 6. SYSTEM SCREEN -->
      <div class="panel-screen" id="screen_system">
        <div style="display: flex; justify-content: space-between; align-items: center;">
          <span style="font-size: 13px; font-weight: 700; color: var(--text);">System Metrics</span>
          <button class="btn-header" onclick="navTo('main_menu')">◀ Back</button>
        </div>
        <div style="font-family: monospace; font-size: 11px; display: flex; flex-direction: column; gap: 6px; background: var(--panel-bg); padding: 10px; border-radius: 6px;">
          <div>Host: mks-skipr-rklipp</div>
          <div>IP: 192.168.1.185</div>
          <div>CPU: 24% | Memory: 1.5 GB / 4.0 GB</div>
          <div>Klipper: v0.12.0-rklipp</div>
          <div>Moonraker: v0.8.0-rust</div>
        </div>
      </div>

    </div>
  </div>

  <div class="diag-panel">
    <span id="touchCoord">Digitizer: (X: ---, Y: ---)</span>
    <span id="lastAction">Status: Ready</span>
  </div>
</div>

<script>
  function navTo(screenId) {
    document.querySelectorAll('.panel-screen').forEach(s => s.classList.remove('active'));
    const target = document.getElementById('screen_' + screenId);
    if (target) {
      target.classList.add('active');
      document.getElementById('screenTitle').innerText = screenId.toUpperCase();
      document.getElementById('lastAction').innerText = "Navigated to " + screenId;
    }
  }

  function createTouchRipple(event) {
    const rect = document.getElementById('touchDisplay').getBoundingClientRect();
    const x = Math.round(event.clientX - rect.left);
    const y = Math.round(event.clientY - rect.top);
    
    document.getElementById('touchCoord').innerText = `Digitizer: (X: ${x}, Y: ${y})`;
    
    const ripple = document.createElement('div');
    ripple.className = 'touch-ripple';
    ripple.style.left = x + 'px';
    ripple.style.top = y + 'px';
    document.getElementById('touchDisplay').appendChild(ripple);
    setTimeout(() => ripple.remove(), 400);
  }

  function setTemps(h, b) {
    document.getElementById('pillHotend').innerText = `${h}° / ${h}°`;
    document.getElementById('pillBed').innerText = `${b}° / ${b}°`;
    document.getElementById('lastAction').innerText = `Set Target: Hotend=${h}°C Bed=${b}°C`;
  }

  function startPrint(name) {
    navTo('job_status');
    document.getElementById('stateBadge').innerText = "PRINTING";
    document.getElementById('stateBadge').style.color = "var(--ok)";
    document.getElementById('lastAction').innerText = "Started print: " + name;
  }

  function triggerEstop() {
    document.getElementById('stateBadge').innerText = "SHUTDOWN";
    document.getElementById('stateBadge').style.color = "var(--hot)";
    document.getElementById('lastAction').innerText = "M112 Emergency Stop Triggered!";
    alert("!! EMERGENCY STOP TRIGGERED (M112) !!");
  }

  function logGcode(cmd) {
    document.getElementById('lastAction').innerText = "G-Code: " + cmd;
  }
</script>
</body>
</html>
"""

def generate_visual_html():
    artifact_dir = "/home/jrad/.gemini/antigravity-ide/brain/d36f1fae-bfea-4b97-9a1c-c9a1139603e2"
    out_file = os.path.join(artifact_dir, "visual_touchscreen_sim.html")
    with open(out_file, "w") as f:
        f.write(HTML_CONTENT)
    print(f"Generated HTML5 Visual Simulator: {out_file}")

if __name__ == "__main__":
    generate_visual_html()
