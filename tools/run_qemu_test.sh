#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# run_qemu_test.sh
# 
# Builds the thumbv7em-none-eabihf target and executes it under QEMU ARM emulation
# (lm3s6965evb / Cortex-M3).
# Captures and displays the serial output, and exits with code 0 ONLY if the log
# contains the string 'TESTS PASSED'.
# ==============================================================================

TARGET="thumbv7em-none-eabihf"
APP_NAME="${1:-your_app}"
TIMEOUT_SEC="${QEMU_TIMEOUT:-15}"

# Check for required tools
if ! command -v qemu-system-arm >/dev/null 2>&1; then
    echo "[-] Error: 'qemu-system-arm' command not found in PATH." >&2
    exit 1
fi

if ! command -v cargo >/dev/null 2>&1; then
    echo "[-] Error: 'cargo' command not found in PATH." >&2
    exit 1
fi

# Determine package name and binary path
if [ "${APP_NAME}" = "klipper-mcu-firmware" ] || [ "${APP_NAME}" = "qemu_runner" ]; then
    PKG_NAME="klipper-mcu-firmware"
    BIN_NAME="qemu_runner"
    KERNEL_PATH="target/${TARGET}/release/${BIN_NAME}"
elif [[ "${APP_NAME}" == */* ]]; then
    PKG_NAME=""
    BIN_NAME=$(basename "${APP_NAME}")
    KERNEL_PATH="${APP_NAME}"
else
    PKG_NAME="${APP_NAME}"
    BIN_NAME="${APP_NAME}"
    KERNEL_PATH="target/${TARGET}/release/${APP_NAME}"
fi

echo "[*] Step 1: Building target '${TARGET}' (release mode)..."
if [ -n "${PKG_NAME}" ]; then
    cargo build --target "${TARGET}" --release -p "${PKG_NAME}" --bin "${BIN_NAME}" || cargo build --target "${TARGET}" --release -p "${PKG_NAME}"
else
    cargo build --target "${TARGET}" --release --bin "${BIN_NAME}" || true
fi

if [ ! -f "${KERNEL_PATH}" ]; then
    echo "[!] Warning: Kernel binary not found at '${KERNEL_PATH}'."
    echo "[!] Searching for existing binaries in target/${TARGET}/release/..."
    FOUND_BIN=$(find "target/${TARGET}/release" -maxdepth 1 -type f -executable ! -name "*.d" ! -name "*.rlib" | head -n 1 || true)
    if [ -n "${FOUND_BIN}" ]; then
        echo "[*] Using discovered binary: ${FOUND_BIN}"
        KERNEL_PATH="${FOUND_BIN}"
    else
        echo "[-] Error: No valid kernel binary found to run under QEMU at '${KERNEL_PATH}'." >&2
        exit 1
    fi
fi

LOG_FILE=$(mktemp /tmp/qemu_serial_XXXXXX.log)
cleanup() {
    rm -f "${LOG_FILE}"
}
trap cleanup EXIT INT TERM

echo "[*] Step 2: Executing QEMU test on '${KERNEL_PATH}' (Timeout: ${TIMEOUT_SEC}s)..."
echo "[*] Command: qemu-system-arm -machine lm3s6965evb -cpu cortex-m3 -nographic -serial mon:stdio -kernel ${KERNEL_PATH}"

# Execute QEMU with timeout, enabling semihosting for output
# Semihosting output goes to stderr, so capture both streams
set +e
timeout "${TIMEOUT_SEC}s" qemu-system-arm \
    -machine lm3s6965evb \
    -cpu cortex-m3 \
    -nographic \
    -semihosting-config enable=on,target=native \
    -kernel "${KERNEL_PATH}" > "${LOG_FILE}" 2>&1
QEMU_EXIT_CODE=$?
set -e

echo ""
echo "==================== QEMU SERIAL OUTPUT ===================="
cat "${LOG_FILE}"
echo "============================================================"
echo ""

# Validate output for 'TESTS PASSED'
if grep -q "TESTS PASSED" "${LOG_FILE}"; then
    echo "[+] SUCCESS: 'TESTS PASSED' detected in serial log."
    exit 0
else
    echo "[-] FAILURE: 'TESTS PASSED' NOT found in serial log (QEMU exit code: ${QEMU_EXIT_CODE})."
    exit 1
fi
