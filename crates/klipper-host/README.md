# Klipper Host Support (`klipper-host`)

This crate provides hardware identification and categorization for systems acting as Klipper hosts. It tracks a variety of popular Single Board Computers (SBCs) and integrated 3D printer control pads to help determine performance capabilities and expectations.

## Supported Hardware

Based on our latest updates, we officially categorize the following hardware architectures and boards:

### Better: Quad-Core Arm Cortex-A72/A76 (1.5 GHz - 2.4 GHz)
*The sweet spot. Effortlessly manages Klipper, web interfaces, and a camera for 2-3 printers.*
* **Raspberry Pi 4 (BCM2711)**
* **Raspberry Pi 5 (BCM2712)**
* **Rock Pi 4**

### Good: Quad-Core Arm Cortex-A53/A7 (1.2 GHz - 1.5 GHz)
*A great starting point. Handles Klipper host duties for 1-2 basic printers easily.*
* **Raspberry Pi 3B**
* **Orange Pi Zero 2**
* **Libre Computer Board AML-S905X-CC**

### Integrated: Quad-Core Arm Cortex-A53 (1.4 GHz)
*An SBC is built right into the control board. This saves space and simplifies wiring.*
* **MKS SKIPR**

### Adequate: Dual-Core MIPS XBurst (1.0 GHz)
*Designed for "turnkey" setups. They run Klipper but have limited power for heavy multitasking.*
* **Creality Nebula Pad**
* **BTT Pad 7**

## Future Perspectives

As the Klipper ecosystem continues to grow, the diversity of host hardware is expanding rapidly. Future updates to this crate will focus on:

1. **Expanding Architecture Support:** 
   While ARM and MIPS are currently well-represented, we plan to track and classify upcoming RISC-V based host boards as they become more viable for Klipper environments.

2. **Granular Capability Mapping:** 
   Beyond just processor speed and core count, future iterations will map out specific hardware capabilities—such as hardware video encoding (for webcams/OctoEverywhere), onboard CAN bus support, and eMMC availability.

3. **Hardware Re-evaluations:** 
   As the resource demands of Klipper interfaces (Mainsail/Fluidd) and extensions (KlipperScreen) evolve, we will continuously re-evaluate and shift boards between the "Adequate", "Good", and "Better" tiers to accurately reflect real-world user experiences.

## Usage

This crate is primarily used internally by the larger workspace to adjust configurations, tailor logging, and optimize resource usage based on the host's identified processor tier.