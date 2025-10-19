# Hardware Setup Guide

This guide provides instructions for setting up the necessary hardware to run the Klipper in Rust firmware on a supported MCU board.

## Supported Hardware

*   **MCU Boards**:
    *   MKS SKIPR (STM32F407VET6) - *Primary target*
    *   (Add other boards as they become supported)
*   **Debug Probes**:
    *   ST-Link v2 / v3
    *   J-Link EDU / Base / Plus
    *   Any other probe supported by `probe-rs`.

## Required Connections

To flash and debug the firmware, you will need to connect a debug probe to your MCU board. This is typically done via the Serial Wire Debug (SWD) interface.

### SWD Port Pinout

The SWD port usually has the following pins:

| Pin   | Description         |
|-------|---------------------|
| `SWDIO` | Serial Wire Data I/O  |
| `SWCLK` | Serial Wire Clock     |
| `GND`   | Ground              |
| `VCC`   | Power (optional)    |

### Example: Connecting to an MKS SKIPR

1.  **Locate the SWD Header**: On the MKS SKIPR board, the SWD header is labeled `SWD`.
2.  **Connect the Probe**:
    *   Connect the `SWDIO` pin from your probe to the `SWDIO` pin on the board.
    *   Connect the `SWCLK` pin from your probe to the `SWCLK` pin on the board.
    *   Connect the `GND` pin from your probe to a `GND` pin on the board.
    *   **Important**: It is recommended to power the board from its own power supply, not from the debug probe, unless you are certain the probe can provide sufficient current.

![MKS SKIPR SWD Connection](https://i.imgur.com/your-connection-diagram.png) <!-- Placeholder for connection diagram -->

## Flashing Procedure

Once the hardware is connected, you can proceed with flashing the firmware.

### Using `cargo-embed`

This is the recommended method for both flashing and debugging.

1.  **Connect the Board**: Connect the debug probe to your computer via USB.
2.  **Run the Command**: Execute the `cargo embed` command with the appropriate feature flag for your board.
    ```bash
    cargo embed --release -p klipper-m-firmware --features mks_skipr
    ```
3.  **Verify**: The tool should automatically detect your probe and the target MCU, flash the firmware, and open a logging terminal.

### Manual Flashing (e.g., for SD Card Update)

Some boards support updating the firmware via an SD card. This requires converting the compiled ELF file to a binary format.

1.  **Build the Firmware**:
    ```bash
    cargo build --release -p klipper-mcu-firmware --features mks_skipr
    ```
2.  **Convert to Binary**: Use `arm-none-eabi-objcopy` to create a `.bin` file.
    ```bash
    arm-none-eabi-objcopy -O binary target/thumbv7em-none-eabihf/release/klipper-mcu-firmware firmware.bin
    ```
3.  **Copy to SD Card**: Copy the `firmware.bin` file to the root of your SD card.
4.  **Install**: Insert the SD card into the MCU board and power it on. The bootloader should automatically flash the new firmware.

## Troubleshooting

*   **Probe Not Detected**:
    *   Ensure the probe is properly connected to your computer.
    *   Check that you have the necessary USB drivers installed (e.g., for ST-Link).
    *   Run `probe-rs-cli info` to see if `probe-rs` can find your probe.
*   **Flashing Fails**:
    *   Verify the SWD connections are correct and secure.
    *   Make sure the MCU is powered on.
    *   Try holding the `BOOT` button on the board while powering it on to put it into DFU (Device Firmware Update) mode, then try flashing again.
*   **Logs Not Appearing**:
    *   Confirm that the firmware was built with `defmt` logging enabled.
    *   Ensure your `Embed.toml` is configured correctly for RTT (Real-Time Transfer).
