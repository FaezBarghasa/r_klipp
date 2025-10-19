# Klipper in Rust: Main MCU Firmware Crate

## Overview

This crate contains the main application logic for the Klipper in Rust MCU firmware. It integrates all the other crates in the workspace to create a complete, monolithic firmware image that runs on the target MCU.

## Responsibilities

*   **Initialization**: Performs all hardware and software initialization at startup, including setting up the clock, peripherals, and memory.
*   **Task Scheduling**: Configures and starts the `embassy` async executor, which manages all the concurrent tasks.
*   **Board-Specific Configuration**: Contains the board-specific code that maps the firmware's abstract hardware requirements to the physical pins and peripherals of a particular MCU board. This is managed via Cargo features.
*   **Top-Level Application Logic**: Ties together the host communication, motion control, and thermal management subsystems into a coherent application.
*   **Panic Handling**: Implements the `panic_handler`, which ensures that the MCU enters a safe state in the event of an unrecoverable error.

## Building and Flashing

To build the firmware for a specific board, you must enable the corresponding feature flag.

```bash
# Build for the MKS SKIPR board
cargo build --release --features mks_skipr

# Flash and debug using cargo-embed
cargo embed --release --features mks_skipr
```

## Adding a New Board

To add support for a new MCU board, you will need to:

1.  **Create a New Board Module**: Add a new module in the `boards/` directory (e.g., `boards/my_new_board.rs`).
2.  **Implement the Board Trait**: In this module, define a struct that implements the `Board` trait, providing the necessary pin mappings and peripheral configurations.
3.  **Add a Cargo Feature**: Create a new feature flag in this crate's `Cargo.toml` file (e.g., `my_new_board`).
4.  **Conditional Compilation**: Use `#[cfg(feature = "my_new_board")]` to ensure that your new board module is only compiled when its feature is enabled.
5.  **Add to `main.rs`**: Add a `use` statement in `main.rs` to include your new board's configuration.
