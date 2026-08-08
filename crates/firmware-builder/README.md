# Klipper in Rust: Firmware Builder Crate

## Overview

The `firmware-builder` crate is a command-line utility or library responsible for compiling the Klipper in Rust firmware with the correct feature flags and build configurations for a specific target board.

It simplifies the build process by abstracting away the underlying `cargo` commands and ensuring that all necessary parameters are set correctly.

## Features

*   **Board Profiles**: Manages build profiles for different MCU boards, specifying the correct target, features, and memory layout.
*   **Configuration Management**: Can be used to apply board-specific configurations at build time.
*   **Artifact Generation**: Handles the conversion of compiled ELF files into other formats, such as `.bin` files for SD card flashing.

## Usage

This tool is intended to be used by developers and CI/CD systems to produce reliable firmware builds.

```bash
# Example usage (hypothetical)
cargo run -p firmware-builder -- --board mks_skipr --output firmware.bin
```
