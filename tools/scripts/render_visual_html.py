#!/usr/bin/env python3
"""
Interactive HTML5 Visual Test & Touchscreen Emulator for Slint GUI (r_klipp)
Renders exact 1-to-1 KlipperScreen Material Dark (480x320 landscape) layout with official styling.
"""

import sys
import os

HTML_CONTENT = """<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="UTF-8">
<meta name="viewport" content="width=device-width, initial-scale=1.0">
<title>KlipperScreen Slint UI - 1:1 Pixel Accurate Simulator</title>
<link href="https://fonts.googleapis.com/css2?family=Roboto:wght@400;500;700&family=JetBrains+Mono:wght@400;600&display=swap" rel="stylesheet">
<style>
  /* KlipperScreen Material Dark official palette */
  :root {
    --ks-bg: #121212;
    --ks-buttons-bg: #090909;
    --ks-panel-bg: #1E1E1E;
    --ks-active-dark: #282828;
    --ks-text: #e2e2e2;
    --ks-text-muted: #a2a2a2;
    --ks-color1: #d46900; /* Orange / Primary */
    --ks-color2: #ed3c63; /* Pink / Secondary */
    --ks-color3: #849900; /* Green / Tertiary */
    --ks-color4: #007db4; /* Blue / Quaternary */
    --ks-active: #d46900;
    --ks-slider: #d81549;
    --ks-echo: #006412;
    --ks-warning: #EF6C00;
    --ks-error: #B71C1C;
    --ks-button-border: #404040;
  }

  * { box-sizing: border-box; margin: 0; padding: 0; user-select: none; }
  body {
    background: #0a0b0e;
    color: var(--ks-text);
    font-family: 'Roboto', sans-serif;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    min-height: 100vh;
    padding: 16px;
  }

  .header-title {
    text-align: center;
    margin-bottom: 16px;
  }
  .header-title h1 {
    font-size: 20px;
    color: var(--ks-color1);
    letter-spacing: 0.5px;
  }
  .header-title p {
    font-size: 12px;
    color: var(--ks-text-muted);
  }

  /* Hardware Enclosure (MKS PI_TS35 3.5" Display) */
  .hardware-case {
    background: linear-gradient(145deg, #24252a, #18191c);
    border: 3px solid #383a42;
    border-radius: 12px;
    padding: 18px 16px 14px 16px;
    box-shadow: 0 16px 40px rgba(0,0,0,0.85);
    display: flex;
    flex-direction: column;
    align-items: center;
  }

  .case-header {
    width: 100%;
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 0 4px 10px 4px;
    font-size: 10px;
    font-weight: 700;
    color: #6e707c;
    letter-spacing: 1.2px;
  }

  /* Exact 480x320 Touch Screen Active Area */
  .touch-display {
    width: 480px;
    height: 320px;
    background: var(--ks-bg);
    border: 1px solid #000;
    border-radius: 4px;
    overflow: hidden;
    position: relative;
    box-shadow: inset 0 0 12px rgba(0,0,0,0.95);
    display: flex;
    flex-direction: column;
  }

  /* KlipperScreen Titlebar */
  .titlebar {
    height: 40px;
    background: var(--ks-buttons-bg);
    border-bottom: 1px solid #202020;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 10px;
  }
  .title-left {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .title-printer {
    font-size: 14px;
    font-weight: 700;
    color: var(--ks-text);
  }
  .title-screen-name {
    font-size: 12px;
    color: var(--ks-color1);
    font-weight: 500;
  }

  .title-right {
    display: flex;
    align-items: center;
    gap: 8px;
  }
  .status-badge {
    font-size: 10px;
    padding: 3px 8px;
    border-radius: 12px;
    font-weight: 700;
    text-transform: uppercase;
    background: #1b5e20;
    color: #fff;
    font-family: 'JetBrains Mono', monospace;
  }
  .clock-display {
    font-size: 13px;
    font-weight: 600;
    color: var(--ks-text);
  }

  /* Main Container (Content + Right Sidebar in Landscape) */
  .main-body {
    flex: 1;
    display: flex;
    overflow: hidden;
  }

  /* Panel Content Viewport */
  .content-area {
    flex: 1;
    height: 280px;
    padding: 8px;
    overflow-y: auto;
    position: relative;
    background: var(--ks-bg);
  }

  /* Right Sidebar Actions (Back, Home, Emergency) */
  .action-sidebar {
    width: 62px;
    height: 280px;
    background: var(--ks-buttons-bg);
    border-left: 1px solid #222;
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 6px 4px;
    gap: 6px;
  }
  .btn-side {
    width: 52px;
    height: 52px;
    background: var(--ks-panel-bg);
    border: 1px solid var(--ks-button-border);
    border-radius: 8px;
    color: var(--ks-text);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: background 0.1s;
    gap: 2px;
  }
  .btn-side:active {
    background: var(--ks-active);
    color: #000;
  }
  .btn-side svg {
    width: 22px;
    height: 22px;
    fill: currentColor;
  }
  .btn-side span {
    font-size: 9px;
    font-weight: 600;
  }
  .btn-estop-side {
    background: #b71c1c;
    border-color: #d32f2f;
    margin-top: auto;
  }
  .btn-estop-side:active { background: #f44336; }

  /* Screens */
  .panel-screen {
    display: none;
    height: 100%;
    flex-direction: column;
    gap: 6px;
  }
  .panel-screen.active {
    display: flex;
  }

  /* 1. Main Menu Screen */
  .heater-bar {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 6px;
    height: 48px;
  }
  .heater-pill {
    background: var(--ks-buttons-bg);
    border: 1px solid var(--ks-button-border);
    border-radius: 8px;
    padding: 4px 8px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    cursor: pointer;
  }
  .heater-pill:active { border-color: var(--ks-color1); background: var(--ks-active-dark); }
  .heater-pill-left {
    display: flex;
    flex-direction: column;
  }
  .heater-pill-title { font-size: 10px; color: var(--ks-text-muted); }
  .heater-pill-val { font-size: 13px; font-weight: 700; color: var(--ks-color1); }

  .menu-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    grid-template-rows: repeat(2, 1fr);
    gap: 6px;
    flex: 1;
    margin-top: 4px;
  }
  .menu-tile {
    background: var(--ks-buttons-bg);
    border: 1px solid var(--ks-button-border);
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 4px;
    cursor: pointer;
    font-size: 11px;
    font-weight: 600;
    color: var(--ks-text);
    transition: transform 0.05s, background 0.1s;
  }
  .menu-tile:active {
    transform: scale(0.96);
    background: var(--ks-active);
    color: #000;
  }
  .menu-tile svg { width: 22px; height: 22px; fill: currentColor; }

  /* 2. Move Screen */
  .move-layout {
    display: grid;
    grid-template-columns: 2fr 1fr;
    gap: 8px;
    height: 100%;
  }
  .jog-cross {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    grid-template-rows: repeat(3, 1fr);
    gap: 4px;
  }
  .btn-jog {
    background: var(--ks-buttons-bg);
    border: 1px solid var(--ks-button-border);
    border-radius: 8px;
    color: var(--ks-text);
    font-weight: 700;
    font-size: 12px;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
  }
  .btn-jog:active { background: var(--ks-color4); color: #fff; }
  .dist-selector {
    display: flex;
    gap: 4px;
    margin-top: 4px;
  }
  .btn-dist {
    flex: 1;
    padding: 6px 0;
    background: var(--ks-buttons-bg);
    border: 1px solid var(--ks-button-border);
    border-radius: 6px;
    font-size: 10px;
    font-weight: 700;
    color: var(--ks-text);
    cursor: pointer;
    text-align: center;
  }
  .btn-dist.active {
    background: var(--ks-color1);
    color: #000;
    border-color: var(--ks-color1);
  }

  /* 3. Temperature Presets */
  .preset-grid {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 6px;
  }
  .preset-tile {
    background: var(--ks-buttons-bg);
    border: 1px solid var(--ks-button-border);
    border-radius: 8px;
    padding: 10px 6px;
    text-align: center;
    cursor: pointer;
  }
  .preset-tile:active { border-color: var(--ks-color1); background: var(--ks-active-dark); }
  .preset-name { font-size: 12px; font-weight: 700; color: var(--ks-text); }
  .preset-temps { font-size: 11px; color: var(--ks-color1); margin-top: 2px; }

  /* 4. Job Status Screen */
  .job-card {
    background: var(--ks-panel-bg);
    border: 1px solid #333;
    border-radius: 8px;
    padding: 10px;
    display: flex;
    flex-direction: column;
    gap: 6px;
  }
  .progress-bar-bg {
    height: 12px;
    background: #252525;
    border-radius: 6px;
    overflow: hidden;
  }
  .progress-bar-fill {
    height: 100%;
    width: 64%;
    background: linear-gradient(90deg, var(--ks-color1), #ff9100);
  }
  .job-meta-row {
    display: flex;
    justify-content: space-between;
    font-size: 10px;
    color: var(--ks-text-muted);
  }

  /* Touch Ripple */
  .touch-ripple {
    position: absolute;
    width: 28px;
    height: 28px;
    border-radius: 50%;
    background: rgba(212, 105, 0, 0.4);
    border: 2px solid var(--ks-color1);
    transform: translate(-50%, -50%) scale(0);
    animation: ripple 0.35s ease-out forwards;
    pointer-events: none;
    z-index: 999;
  }
  @keyframes ripple {
    0% { transform: translate(-50%, -50%) scale(0.2); opacity: 1; }
    100% { transform: translate(-50%, -50%) scale(1.4); opacity: 0; }
  }

  /* Diagnostic Footer */
  .diag-panel {
    margin-top: 12px;
    width: 480px;
    background: #141519;
    border: 1px solid #282a32;
    border-radius: 6px;
    padding: 8px 12px;
    font-family: 'JetBrains Mono', monospace;
    font-size: 11px;
    color: #8c909a;
    display: flex;
    justify-content: space-between;
  }
</style>
</head>
<body>

<div class="header-title">
  <h1>KlipperScreen Slint UI Touchscreen Simulator</h1>
  <p>1:1 Exact Parity with references/KlipperScreen (Material-Dark Theme)</p>
</div>

<div class="hardware-case">
  <div class="case-header">
    <span>MKS SKIPR HOST</span>
    <span>480×320 PI_TS35 DISPLAY</span>
  </div>

  <div class="touch-display" id="touchDisplay" onclick="createTouchRipple(event)">
    <!-- Top Titlebar -->
    <div class="titlebar">
      <div class="title-left">
        <span class="title-printer">VORON 2.4</span>
        <span class="title-screen-name" id="screenTitle">| MAIN</span>
      </div>
      <div class="title-right">
        <span class="status-badge" id="stateBadge">READY</span>
        <span class="clock-display" id="clockText">12:45 PM</span>
      </div>
    </div>

    <!-- Main Body with Landscape Right Action Bar -->
    <div class="main-body">
      
      <!-- Content Viewport -->
      <div class="content-area">
        
        <!-- 1. MAIN MENU SCREEN -->
        <div class="panel-screen active" id="screen_main">
          <div class="heater-bar">
            <div class="heater-pill" onclick="navTo('temperature')">
              <div class="heater-pill-left">
                <span class="heater-pill-title">Extruder</span>
                <span class="heater-pill-val" id="valExtruder">215.5° / 220°</span>
              </div>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="var(--ks-color1)"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/></svg>
            </div>
            <div class="heater-pill" onclick="navTo('temperature')">
              <div class="heater-pill-left">
                <span class="heater-pill-title">Heater Bed</span>
                <span class="heater-pill-val" id="valBed">60.2° / 65°</span>
              </div>
              <svg width="18" height="18" viewBox="0 0 24 24" fill="var(--ks-color4)"><path d="M19 3H5c-1.1 0-2 .9-2 2v14c0 1.1.9 2 2 2h14c1.1 0 2-.9 2-2V5c0-1.1-.9-2-2-2zm-2 10H7v-2h10v2z"/></svg>
            </div>
          </div>

          <div class="menu-grid">
            <div class="menu-tile" onclick="navTo('move')">
              <svg viewBox="0 0 24 24"><path d="M10 9h4V6h3l-5-5-5 5h3v3zm-1 1H6V7l-5 5 5 5v-3h3v-4zm14 2l-5-5v3h-3v4h3v3l5-5zm-9 3h-4v3H7l5 5 5-5h-3v-3z"/></svg>
              <span>Move</span>
            </div>
            <div class="menu-tile" onclick="navTo('temperature')">
              <svg viewBox="0 0 24 24"><path d="M15 13V5c0-1.66-1.34-3-3-3S9 3.34 9 5v8c-1.21.91-2 2.37-2 4 0 2.76 2.24 5 5 5s5-2.24 5-5c0-1.63-.79-3.09-2-4zm-4-8c0-.55.45-1 1-1s1 .45 1 1h-2z"/></svg>
              <span>Temperature</span>
            </div>
            <div class="menu-tile" onclick="navTo('extrude')">
              <svg viewBox="0 0 24 24"><path d="M19 9h-4V3H9v6H5l7 7 7-7zM5 18v2h14v-2H5z"/></svg>
              <span>Extrude</span>
            </div>
            <div class="menu-tile" onclick="navTo('fan')">
              <svg viewBox="0 0 24 24"><path d="M12 12c0-3 2.5-5.5 5.5-5.5S23 9 23 12h-2c0-1.9-1.6-3.5-3.5-3.5S14 10.1 14 12h-2zm-8.5 0C3.5 9 6 6.5 9 6.5S14.5 9 14.5 12h-2C12.5 10.1 10.9 8.5 9 8.5S5.5 10.1 5.5 12h-2z"/></svg>
              <span>Fans</span>
            </div>
            <div class="menu-tile" onclick="navTo('files')">
              <svg viewBox="0 0 24 24"><path d="M10 4H4c-1.1 0-1.99.9-1.99 2L2 18c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V8c0-1.1-.9-2-2-2h-8l-2-2z"/></svg>
              <span>Print Files</span>
            </div>
            <div class="menu-tile" onclick="navTo('job_status')">
              <svg viewBox="0 0 24 24"><path d="M11.99 2C6.47 2 2 6.48 2 12s4.47 10 9.99 10C17.52 22 22 17.52 22 12S17.52 2 11.99 2zM12 20c-4.42 0-8-3.58-8-8s3.58-8 8-8 8 3.58 8 8-3.58 8-8 8zm.5-13H11v6l5.25 3.15.75-1.23-4.5-2.67z"/></svg>
              <span>Job Status</span>
            </div>
            <div class="menu-tile" onclick="navTo('bed_mesh')">
              <svg viewBox="0 0 24 24"><path d="M20 2H4c-1.1 0-2 .9-2 2v16c0 1.1.9 2 2 2h16c1.1 0 2-.9 2-2V4c0-1.1-.9-2-2-2zm-9 18H4v-7h7v7zm0-9H4V4h7v7zm9 9h-7v-7h7v7zm0-9h-7V4h7v7z"/></svg>
              <span>Bed Mesh</span>
            </div>
            <div class="menu-tile" onclick="navTo('system')">
              <svg viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-6h2v6zm0-8h-2V7h2v2z"/></svg>
              <span>System</span>
            </div>
          </div>
        </div>

        <!-- 2. MOVE SCREEN -->
        <div class="panel-screen" id="screen_move">
          <div class="move-layout">
            <div class="jog-cross">
              <div></div>
              <div class="btn-jog" onclick="logAction('G1 Y+10')">Y+</div>
              <div></div>
              <div class="btn-jog" onclick="logAction('G1 X-10')">X-</div>
              <div class="btn-jog" style="background: var(--ks-panel-bg); color: var(--ks-color1);" onclick="logAction('G28')">G28</div>
              <div class="btn-jog" onclick="logAction('G1 X+10')">X+</div>
              <div></div>
              <div class="btn-jog" onclick="logAction('G1 Y-10')">Y-</div>
              <div></div>
            </div>
            <div style="display: flex; flex-direction: column; gap: 4px;">
              <div class="btn-jog" style="height: 42px;" onclick="logAction('G1 Z+5')">Z+</div>
              <div class="btn-jog" style="height: 42px;" onclick="logAction('G28 Z')">Z Home</div>
              <div class="btn-jog" style="height: 42px;" onclick="logAction('G1 Z-5')">Z-</div>
              <div class="btn-jog" style="height: 32px; background: #263238; font-size: 10px;" onclick="logAction('M84')">Motors Off</div>
            </div>
          </div>
          <div class="dist-selector">
            <div class="btn-dist" onclick="setDist(this, 0.1)">0.1mm</div>
            <div class="btn-dist" onclick="setDist(this, 1)">1mm</div>
            <div class="btn-dist active" onclick="setDist(this, 10)">10mm</div>
            <div class="btn-dist" onclick="setDist(this, 50)">50mm</div>
            <div class="btn-dist" onclick="setDist(this, 100)">100mm</div>
          </div>
        </div>

        <!-- 3. TEMPERATURE SCREEN -->
        <div class="panel-screen" id="screen_temperature">
          <div style="font-size: 12px; font-weight: 700; color: var(--ks-color1); margin-bottom: 4px;">Material Presets</div>
          <div class="preset-grid">
            <div class="preset-tile" onclick="setTemps(205, 60)">
              <div class="preset-name">PLA</div>
              <div class="preset-temps">205° / 60°</div>
            </div>
            <div class="preset-tile" onclick="setTemps(240, 80)">
              <div class="preset-name">PETG</div>
              <div class="preset-temps">240° / 80°</div>
            </div>
            <div class="preset-tile" onclick="setTemps(255, 105)">
              <div class="preset-name">ABS</div>
              <div class="preset-temps">255° / 105°</div>
            </div>
            <div class="preset-tile" onclick="setTemps(0, 0)" style="grid-column: span 3; border-color: var(--ks-color4);">
              <div class="preset-name" style="color: var(--ks-color4);">COOLDOWN ALL (0°C)</div>
            </div>
          </div>
        </div>

        <!-- 4. JOB STATUS SCREEN -->
        <div class="panel-screen" id="screen_job_status">
          <div style="font-size: 12px; font-weight: 700; color: var(--ks-color1);">voron_cube_v2.gcode</div>
          <div class="job-card">
            <div style="display: flex; justify-content: space-between; font-size: 11px;">
              <span>Progress: 64%</span>
              <span>Layer: 78 / 120</span>
            </div>
            <div class="progress-bar-bg">
              <div class="progress-bar-fill"></div>
            </div>
            <div class="job-meta-row">
              <span>Speed: 105%</span>
              <span>Time Left: 18m</span>
              <span>Flow: 100%</span>
            </div>
          </div>
          <div style="display: flex; gap: 6px; margin-top: 4px;">
            <div class="btn-jog" style="flex: 1; height: 36px;" onclick="logAction('PAUSE')">Pause</div>
            <div class="btn-jog" style="flex: 1; height: 36px; background: #b71c1c; border-color: #d32f2f;" onclick="logAction('CANCEL_PRINT')">Cancel</div>
          </div>
        </div>

        <!-- 5. FILES SCREEN -->
        <div class="panel-screen" id="screen_files">
          <div style="font-size: 12px; font-weight: 700; color: var(--ks-color1); margin-bottom: 4px;">G-Code Storage</div>
          <div style="display: flex; flex-direction: column; gap: 4px;">
            <div class="preset-tile" style="display: flex; justify-content: space-between; padding: 8px;" onclick="startPrint('voron_cube_v2.gcode')">
              <span>🗎 voron_cube_v2.gcode</span>
              <span style="color: var(--ks-text-muted);">1.2 MB</span>
            </div>
            <div class="preset-tile" style="display: flex; justify-content: space-between; padding: 8px;" onclick="startPrint('3dbenchy_pla.gcode')">
              <span>🗎 3dbenchy_pla.gcode</span>
              <span style="color: var(--ks-text-muted);">4.5 MB</span>
            </div>
          </div>
        </div>

        <!-- 6. SYSTEM SCREEN -->
        <div class="panel-screen" id="screen_system">
          <div style="font-size: 12px; font-weight: 700; color: var(--ks-color1); margin-bottom: 4px;">System Metrics</div>
          <div style="background: var(--ks-buttons-bg); border: 1px solid var(--ks-button-border); border-radius: 6px; padding: 8px; font-family: monospace; font-size: 10px; display: flex; flex-direction: column; gap: 4px;">
            <div>Host: mks-skipr-rklipp</div>
            <div>IP: 192.168.1.185</div>
            <div>CPU: 24% | Memory: 1.5 GB / 4.0 GB</div>
            <div>Klipper: v0.12.0-rklipp</div>
            <div>Moonraker: v0.8.0-rust</div>
          </div>
        </div>

        <!-- 7. BED MESH SCREEN -->
        <div class="panel-screen" id="screen_bed_mesh">
          <div style="font-size: 12px; font-weight: 700; color: var(--ks-color1); margin-bottom: 4px;">Bed Mesh Calibration (5×5)</div>
          <div style="display: grid; grid-template-columns: repeat(5, 1fr); gap: 4px; flex: 1;">
            <div style="background: #1e3d2f; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 9px;">+0.01</div>
            <div style="background: #2e5d3f; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 9px;">+0.02</div>
            <div style="background: #1e3d2f; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 9px;">0.00</div>
            <div style="background: #3e2d2f; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 9px;">-0.02</div>
            <div style="background: #4e1d1f; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 9px;">-0.04</div>
            <div style="background: #2e5d3f; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 9px;">+0.03</div>
            <div style="background: #3e7d4f; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 9px;">+0.05</div>
            <div style="background: #2e5d3f; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 9px;">+0.02</div>
            <div style="background: #1e3d2f; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 9px;">0.00</div>
            <div style="background: #3e2d2f; border-radius: 4px; display: flex; align-items: center; justify-content: center; font-size: 9px;">-0.02</div>
          </div>
          <div class="btn-jog" style="height: 32px; margin-top: 4px;" onclick="logAction('BED_MESH_CALIBRATE')">Calibrate Mesh</div>
        </div>

      </div>

      <!-- Right Action Sidebar (Exact KlipperScreen landscape layout) -->
      <div class="action-sidebar">
        <div class="btn-side" onclick="navTo('main')">
          <svg viewBox="0 0 24 24"><path d="M11.67 3.87L9.9 2.1 0 12l9.9 9.9 1.77-1.77L3.54 12z"/></svg>
          <span>Back</span>
        </div>
        <div class="btn-side" onclick="navTo('main')">
          <svg viewBox="0 0 24 24"><path d="M10 20v-6h4v6h5v-8h3L12 3 2 12h3v8z"/></svg>
          <span>Home</span>
        </div>
        <div class="btn-side btn-estop-side" onclick="triggerEstop()">
          <svg viewBox="0 0 24 24"><path d="M12 2C6.48 2 2 6.48 2 12s4.48 10 10 10 10-4.48 10-10S17.52 2 12 2zm1 15h-2v-2h2v2zm0-4h-2V7h2v6z"/></svg>
          <span>E-STOP</span>
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
      document.getElementById('screenTitle').innerText = "| " + screenId.toUpperCase();
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
    setTimeout(() => ripple.remove(), 350);
  }

  function setDist(el, d) {
    document.querySelectorAll('.btn-dist').forEach(b => b.classList.remove('active'));
    el.classList.add('active');
    document.getElementById('lastAction').innerText = `Jog distance set to ${d}mm`;
  }

  function setTemps(h, b) {
    document.getElementById('valExtruder').innerText = `${h}° / ${h}°`;
    document.getElementById('valBed').innerText = `${b}° / ${b}°`;
    document.getElementById('lastAction').innerText = `Target set: Extruder=${h}°C Bed=${b}°C`;
  }

  function startPrint(name) {
    navTo('job_status');
    document.getElementById('stateBadge').innerText = "PRINTING";
    document.getElementById('stateBadge').style.background = "#2e7d32";
    document.getElementById('lastAction').innerText = "Started print: " + name;
  }

  function triggerEstop() {
    document.getElementById('stateBadge').innerText = "SHUTDOWN";
    document.getElementById('stateBadge').style.background = "#b71c1c";
    document.getElementById('lastAction').innerText = "M112 Emergency Stop Triggered!";
    alert("!! EMERGENCY STOP TRIGGERED (M112) !!");
  }

  function logAction(cmd) {
    document.getElementById('lastAction').innerText = "Executed: " + cmd;
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
    print(f"Generated 1:1 KlipperScreen Visual Simulator: {out_file}")

if __name__ == "__main__":
    generate_visual_html()
