# Flashing Firmware via Debug Probe

Using a debug probe (like an ST-Link or J-Link) is the recommended method for flashing the firmware during development. It allows you to flash and debug the firmware without needing to use an SD card.

## Prerequisites

*   A debug probe (e.g., ST-Link V2).
*   `cargo-embed` installed on your computer.
*   Physical access to the SWD pins on the MKS SKIPR board.

## SWD Pinout

You will need to connect the following pins from your debug probe to the MKS SKIPR board:

*   `SWDIO`
*   `SWCLK`
*   `GND`
*   `3.3V` (optional, if powering the board through the probe)

Please consult the MKS SKIPR schematic to locate these pins.

## Flashing with `cargo-embed`

The recommended tool for flashing the firmware is `cargo-embed`, which is part of the `probe-rs` toolkit.

### 1. Install `cargo-embed`

If you haven't already, install `cargo-embed` with the following command:

```bash
cargo install cargo-embed
```

### 2. Configure `Embed.toml`

The `klipper-mcu-firmware` crate contains an `Embed.toml` file that configures `cargo-embed` for the MKS SKIPR board. You can customize this file if you need to change the default settings.

### 3. Build and Flash

To build and flash the firmware in one step, run the following command from the root of the repository:

```bash
cargo embed --release -p klipper-mcu-firmware --features mks_skipr
```

This command will:

1.  Compile the firmware in release mode.
2.  Connect to the debug probe.
3.  Flash the firmware to the MCU.
4.  Start a debugging session.

After flashing, the new firmware will be running on the board, and you can interact with it through the debugger.