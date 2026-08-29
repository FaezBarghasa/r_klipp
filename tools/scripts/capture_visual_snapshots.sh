#!/usr/bin/env bash
set -e

ARTIFACT_DIR="/home/jrad/.gemini/antigravity-ide/brain/d36f1fae-bfea-4b97-9a1c-c9a1139603e2"
SNAPSHOT_DIR="${ARTIFACT_DIR}/snapshots"
mkdir -p "${SNAPSHOT_DIR}"

SCREENS=(
    "main_menu"
    "move"
    "temperature"
    "files"
    "job_status"
    "fine_tune"
    "bed_level"
    "bed_mesh"
    "console"
    "system"
    "settings"
    "shutdown"
)

echo "=================================================="
echo " Starting Visual Screenshot Capture for Slint GUI"
echo " Target Resolution: 480x320 (MKS PI_TS35 Display)"
echo " Snapshot Output: ${SNAPSHOT_DIR}"
echo "=================================================="

for screen in "${SCREENS[@]}"; do
    echo "Capturing screen: ${screen}..."
    OUT_PNG="${SNAPSHOT_DIR}/snapshot_${screen}.png"
    
    # Run the Slint GUI inside virtual X11 buffer with dynamic DISPLAY
    xvfb-run -a --server-args="-screen 0 480x320x24" bash -c "
        export SLINT_BACKEND=software
        export LIBGL_ALWAYS_SOFTWARE=1
        ./target/debug/examples/snapshot_screens '${screen}' &
        PID=\$!
        sleep 0.5
        ffmpeg -y -f x11grab -video_size 480x320 -i \"\${DISPLAY}\" -frames:v 1 -update 1 '${OUT_PNG}' > /dev/null 2>&1
        kill -9 \$PID > /dev/null 2>&1 || true
        wait \$PID 2>/dev/null || true
    "
    sleep 0.2
    if [ -f "${OUT_PNG}" ] && [ $(stat -c%s "${OUT_PNG}") -gt 1000 ]; then
        echo "  ✔ Captured: ${OUT_PNG} ($(stat -c%s "${OUT_PNG}") bytes)"
    else
        echo "  ⚠ Captured (${OUT_PNG}): $(stat -c%s "${OUT_PNG}" 2>/dev/null || echo 0) bytes"
    fi
done

echo "=================================================="
echo " Visual Screenshot Capture Complete!"
echo "=================================================="
