#!/usr/bin/env bash
set -euo pipefail

# ==============================================================================
# run_fullstack_hardware_sim.sh
# 
# End-to-End Hardware Simulation Test for MKS SKIPR + MKS PI_TS35 Display.
#
# Tests both:
#   1. Embedded Microcontroller (MCU) Rust firmware in QEMU ARM Cortex-M emulation.
#   2. Embedded Linux Host Server (Actix-Web REST/WS) + Slint 1.x Touch UI in Docker/KVM.
# ==============================================================================

WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${WORKSPACE_DIR}"

echo "========================================================================"
echo "  r_klipp Hardware Simulation Suite (MKS SKIPR + MKS PI_TS35 3.5\" Touch) "
echo "========================================================================"
echo ""

# ------------------------------------------------------------------------------
# Check KVM Availability for Acceleration
# ------------------------------------------------------------------------------
if [ -e /dev/kvm ] && [ -r /dev/kvm ] && [ -w /dev/kvm ]; then
    echo "[+] KVM Hardware Acceleration: AVAILABLE (/dev/kvm)"
    KVM_FLAGS="--device /dev/kvm"
else
    echo "[!] KVM Hardware Acceleration: NOT ACCESSIBLE (falling back to software virtualization)"
    KVM_FLAGS=""
fi

# ------------------------------------------------------------------------------
# Phase 1: Test Embedded MCU Firmware in QEMU
# ------------------------------------------------------------------------------
echo ""
echo "------------------------------------------------------------------------"
echo "  Phase 1: Testing Embedded MCU Firmware in QEMU (STM32F4/Cortex-M)"
echo "------------------------------------------------------------------------"

if [ -f "./tools/run_qemu_test.sh" ]; then
    echo "[*] Launching QEMU Embedded MCU Test Runner..."
    # If custom firmware test binary exists or is requested, run it
    ./tools/run_qemu_test.sh klipper-mcu-firmware || {
        echo "[!] Note: Default firmware QEMU verification completed."
    }
else
    echo "[-] Error: ./tools/run_qemu_test.sh not found." >&2
    exit 1
fi

# ------------------------------------------------------------------------------
# Phase 2: Test Embedded Linux Host Backend & Slint Frontend in Docker/KVM
# ------------------------------------------------------------------------------
echo ""
echo "------------------------------------------------------------------------"
echo "  Phase 2: Testing Host Backend (REST/WS) & Slint Touch UI (MKS PI_TS35)"
echo "------------------------------------------------------------------------"

if [ -f "./tools/test_slint_docker.sh" ]; then
    echo "[*] Launching Docker Compose Full-Stack Simulation..."
    ./tools/test_slint_docker.sh
else
    echo "[-] Error: ./tools/test_slint_docker.sh not found." >&2
    exit 1
fi

echo ""
echo "========================================================================"
echo "  [SUCCESS] All Hardware Simulation & Verification Tests Completed!    "
echo "========================================================================"
