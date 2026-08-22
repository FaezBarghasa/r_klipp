# Hardware Setup & Flashing Guide

This guide provides instructions for wiring, flashing, and debugging `r_klipp` firmware on supported microcontroller boards.

---

## 🔌 1. Supported Microcontroller Boards

| Board | Microcontroller | Architecture | Typical Interfaces |
| :--- | :--- | :--- | :--- |
| **MKS SKIPR** | STM32F407VET6 | ARM Cortex-M4F @ 168 MHz | USB-C, CAN-FD, SPI Steppers, TS35 Display |
| **BTT Octopus / Pro** | STM32F446 / STM32H723 | ARM Cortex-M4/M7 | USB-C, Dual CAN-FD, 8x Stepper Drivers |
| **Raspberry Pi Pico / Pico 2** | RP2040 / RP2350 | Dual ARM Cortex-M0+/M33 | USB Micro/Type-C, PIO Steppers, I2C/SPI |
| **Toolhead / Feeder Boards** | STM32F072 / RP2040 | ARM Cortex-M0+ | CAN-FD Transceiver, TMC2209/2240 |

---

## 🛠️ 2. Hardware Debug Probe Connection (SWD)

To flash and debug firmware with zero-latency logging, connect a hardware debug probe (ST-Link v2/v3, J-Link, or CMSIS-DAP / Raspberry Pi Debug Probe) via Serial Wire Debug (SWD):

### SWD Pinout Table
| Pin | Description | Board Header Pin |
| :--- | :--- | :--- |
| **`SWDIO`** | Serial Wire Data I/O | SWDIO / PA13 |
| **`SWCLK`** | Serial Wire Clock | SWCLK / PA14 |
| **`GND`** | Ground Reference | GND |
| **`VCC`** | Target Voltage Sense (3.3V) | 3.3V (Sense only) |

> [!IMPORTANT]
> Always power the mainboard using its dedicated 12V/24V power supply or USB power before initiating flashing with a probe.

---

## ⚡ 3. Compilation & Flashing Procedures

### 3.1 Flashing via `probe-rs` (Recommended)
`probe-rs` provides automated detection, flashing, and RTT terminal logging:

```bash
# Build and flash directly to STM32F407
probe-rs run --chip STM32F407VETx target/thumbv7em-none-eabihf/release/klipper-mcu-firmware
```

### 3.2 Flashing via DFU (USB Bootloader)
1. Hold the board's `BOOT0` button and press `RESET` to enter STM32 DFU mode.
2. Convert the firmware ELF to binary:
   ```bash
   cargo objcopy -p klipper-mcu-firmware --release --target thumbv7em-none-eabihf -- -O binary target/firmware.bin
   ```
3. Flash using `dfu-util`:
   ```bash
   dfu-util -a 0 -s 0x08000000:leave -D target/firmware.bin
   ```

### 3.3 Flashing via SD Card Bootloader (MKS SKIPR)
1. Build the release binary:
   ```bash
   cargo build --release -p klipper-mcu-firmware --target thumbv7em-none-eabihf --features embassy-rt
   ```
2. Copy binary to FAT32 SD card renamed as `mks_skipr.bin`.
3. Insert SD card into the board's SD slot and power cycle. The on-board bootloader flashes the image in ~10 seconds.
