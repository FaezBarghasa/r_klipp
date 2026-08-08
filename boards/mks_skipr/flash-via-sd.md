# Flashing Firmware via SD Card

This is the simplest method for updating the firmware on your MKS SKIPR board, and it does not require a debug probe.

## Prerequisites

*   A microSD card (<= 32GB), formatted as FAT32.
*   `arm-none-eabi-objcopy` installed on your computer. This is part of the `arm-none-eabi-gcc` toolchain.

## Steps

### 1. Build the Firmware

First, build the firmware in release mode:

```bash
cargo build --release -p klipper-mcu-firmware --features mks_skipr
```

The compiled ELF file will be located at `target/thumbv7em-none-eabihf/release/klipper-mcu-firmware`.

### 2. Create the Binary File

Next, use `arm-none-eabi-objcopy` to convert the ELF file to a binary file:

```bash
arm-none-eabi-objcopy -O binary \
    target/thumbv7em-none-eabihf/release/klipper-mcu-firmware \
    firmware.bin
```

This will create a `firmware.bin` file in the root of the repository.

### 3. Prepare the SD Card

1.  Copy the `firmware.bin` file to the root directory of your microSD card.
2.  Rename the file on the SD card to `mks_skipr.bin`.
3.  Safely eject the SD card from your computer.

### 4. Flash the Board

1.  Ensure the MKS SKIPR is powered off.
2.  Insert the microSD card into the board.
3.  Power on the board.

## Verification

The board's status LEDs may blink during the update process. After a few moments, the firmware update will complete. The board will rename the file on the SD card to `mks_skipr.cur`. You can verify this by inserting the SD card back into your computer. The board will now be running the new firmware.