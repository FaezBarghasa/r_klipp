# Klipper in Rust: Motion Control Crate

## Overview

The `motion` crate is responsible for all motion-related tasks in the Klipper in Rust firmware. This includes planning and executing movements, generating step pulses, and managing the kinematics of the printer.

This is a `no_std` library, designed for high-performance, real-time execution in an embedded environment.

## Features

*   **Trapezoidal Motion Planning**: Implements a trapezoidal velocity profile generator, which is the standard for 3D printer motion control.
*   **Step Pulse Generation**: Contains a highly optimized step generation algorithm that can produce precise and jitter-free pulse trains for the stepper motors.
*   **Kinematics Support**: Includes a pluggable kinematics system, allowing it to support different printer geometries, such as:
    *   Cartesian (the most common type)
    *   CoreXY
    *   Delta
*   **Motion Queue**: Manages a queue of motion blocks received from the Klipper host, ensuring that the motion system never starves for data.
*   **Lookahead**: Implements a lookahead mechanism to smooth out the motion at the corners of segments, allowing for higher printing speeds.

## Design

The motion control system is designed to be highly deterministic and performant.

*   **Interrupt-Driven**: The core step generation loop is designed to be run from a high-priority hardware timer interrupt. This ensures that step pulses are generated with precise timing, even when the main application is busy with other tasks.
*   **Fixed-Point Arithmetic**: The motion planning calculations use fixed-point arithmetic to avoid the overhead and non-determinism of floating-point operations in a real-time context.
*   **Low Latency**: The entire motion pipeline, from receiving a command to generating the first step pulse, is optimized for low latency.

## Usage

The `motion` crate is used by the main `klipper-mcu-firmware` crate. The main application feeds motion commands into the motion planner, which then drives the stepper motors via the `mcu-drivers` crate.
