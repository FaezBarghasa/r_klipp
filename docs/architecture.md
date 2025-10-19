# Architecture Overview

This document provides a high-level overview of the Klipper Rust firmware architecture. The system is designed to be modular, safe, and performant, leveraging the strengths of the Rust programming language.

## Host-MCU Split

Klipper's architecture is split into two main components:

1.  **Host Software (`klipper-host`)**: A high-level process that runs on a general-purpose computer (like a Raspberry Pi). It is responsible for:
    *   Parsing G-code files.
    *   Handling motion planning and kinematics.
    *   Managing the user interface (e.g., web interface).
    *   Sending commands to the MCU.

2.  **MCU Firmware (`klipper-mcu-firmware`)**: A real-time firmware that runs on the 3D printer's microcontroller (MCU). It is responsible for:
    *   Executing commands received from the host.
    *   Controlling stepper motors with precise timing.
    *   Monitoring sensors (thermistors, endstops).
    *   Managing heaters and fans.

This split allows the computationally intensive tasks to be handled by the more powerful host computer, while the MCU focuses on real-time control.

## Workspace Crates

The project is organized as a Cargo workspace, with each crate having a specific responsibility:

*   **`klipper-mcu-firmware`**: The main firmware application that runs on the MCU. It integrates all the other `no_std` crates into a single binary.

*   **`klipper-host`**: The host-side software that runs on a general-purpose computer. It communicates with the MCU and provides a user interface.

*   **`klipper-proto`**: Defines the communication protocol between the host and the MCU. It provides serialization and deserialization for all commands and responses.

*   **`motion`**: Contains the motion control logic, including stepper pulse generation and motion planning.

*   **`mcu-drivers`**: A collection of drivers for various MCU peripherals and external components, such as stepper motor drivers and thermistors.

*   **`compat-layer`**: A compatibility layer for interfacing with C-based components of the Klipper ecosystem.

*   **`sim`**: A simulation environment for running and testing the firmware on a host machine without requiring physical hardware.

## Communication Protocol

The host and MCU communicate over a serial connection using a custom binary protocol defined in the `klipper-proto` crate. The protocol is designed to be efficient and reliable, with features like command compression and checksums to ensure data integrity.

## Real-Time Schedulers

The `klipper-mcu-firmware` can be compiled with one of two real-time schedulers:

*   **Embassy**: An `async/await` framework for writing concurrent, non-blocking code. This is the default scheduler.
*   **RTIC (Real-Time For the Impatient)**: A framework for building real-time systems with predictable scheduling and minimal overhead.

The choice of scheduler can be configured at compile time, allowing the firmware to be adapted to different hardware and performance requirements.