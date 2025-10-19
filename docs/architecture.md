# Klipper in Rust: System Architecture

This document provides a high-level overview of the architecture for the Klipper MCU firmware written in Rust. It is intended for developers who want to understand the design principles, component interactions, and overall structure of the firmware.

## Core Principles

The firmware is designed around the following core principles:

1.  **Safety and Reliability**: Leveraging Rust's compile-time guarantees to prevent common embedded systems bugs (e.g., race conditions, buffer overflows, null pointer dereferencing). A multi-layered safety system is implemented to handle runtime faults.
2.  **Asynchronous Execution**: Using the `embassy` framework for cooperative multitasking. This allows for efficient, non-blocking handling of I/O and concurrent tasks without the overhead of a traditional RTOS.
3.  **Modularity and Portability**: The codebase is organized into modular crates with clear interfaces. A Hardware Abstraction Layer (HAL) isolates hardware-specific code, making it easier to port the firmware to different MCUs.
4.  **Performance**: Critical paths, such as the stepper motor control loop, are highly optimized for low-latency and deterministic execution.

## System Components

The firmware is composed of several key components that work together to control the 3D printer.

![System Diagram](https://i.imgur.com/your-diagram.png)  <!-- Placeholder for a future diagram -->

### 1. Host Communication Interface

*   **Description**: This component manages the communication link (typically USB or UART) with the Klipper host software.
*   **Responsibilities**:
    *   Receiving and deserializing commands from the host.
    *   Sending responses and sensor data back to the host.
    *   Maintaining protocol synchronization.
*   **Implementation**: It uses an async, non-blocking reader and writer task that interfaces with the MCU's serial peripheral. The `klipper-proto` crate defines the data structures and serialization logic.

### 2. Command Dispatcher

*   **Description**: The central hub that routes incoming commands to the appropriate subsystems.
*   **Responsibilities**:
    *   Parsing the command queue from the host interface.
    *   Validating command parameters.
    *   Calling the corresponding functions in other modules (e.g., `motion`, `thermal`).
*   **Implementation**: A state machine that reads from a command buffer and executes handlers based on the command ID.

### 3. Motion Control Subsystem

*   **Description**: Responsible for all printer movements. This is the most timing-critical part of the firmware.
*   **Responsibilities**:
    *   Managing a queue of trapezoidal motion blocks.
    *   Generating precise step pulses for the stepper motors.
    *   Monitoring endstops and triggering motion halts.
*   **Implementation**:
    *   It uses a dedicated high-priority task or interrupt handler.
    *   The stepper pulse generation relies on hardware timers (e.g., PWM or general-purpose timers) to ensure jitter-free pulse trains.
    *   The motion queue is carefully managed to ensure it never runs dry during a print, which would cause stuttering.

### 4. Thermal Management Subsystem

*   **Description**: Manages the temperature of heaters (e.g., hotend, bed) and monitors thermistors.
*   **Responsibilities**:
    *   Periodically sampling ADC channels connected to thermistors.
    *   Converting ADC readings to temperature values.
    *   Implementing a PID control loop to regulate heater PWM output.
    *   Enforcing thermal safety limits.
*   **Implementation**:
    *   An async task runs at a fixed interval (e.g., every 100ms) to sample temperatures.
    *   The PID controller updates the PWM duty cycle for each heater based on the target and current temperatures.
    *   See `docs/safety.md` for details on the thermal safety features.

### 5. Hardware Abstraction Layer (HAL)

*   **Description**: An abstraction layer that provides a consistent API for interacting with MCU peripherals.
*   **Responsibilities**:
    *   Wrapping low-level register access in safe, high-level Rust interfaces.
    *   Providing drivers for peripherals like GPIO, UART, ADC, PWM, SPI, and I2C.
*   **Implementation**:
    *   The `mcu-drivers` crate contains HAL implementations for different MCU families (e.g., `stm32f4xx-hal`).
    *   The main firmware code is written against the traits and types defined by `embedded-hal`, making it portable.

### 6. Safety and Monitoring Subsystem

*   **Description**: A dedicated subsystem that monitors the state of the printer and intervenes if an unsafe condition is detected.
*   **Responsibilities**:
    *   **Watchdog**: A hardware watchdog is continuously reset. If the main loop hangs, the watchdog will reset the MCU.
    *   **Thermal Runaway Protection**: Monitors heaters to ensure they are heating up as expected and shuts them down if they exceed temperature limits or fail to reach the target.
    *   **Sensor Failure Detection**: Detects disconnected or shorted thermistors.
    *   **Emergency Stop**: Provides a mechanism to immediately halt all motion and disable all heaters.
*   **Implementation**: A high-priority async task that runs checks at regular intervals. It has the authority to override other subsystems and put the machine into a safe state.

## Concurrency and Scheduling

The firmware uses the `embassy` executor to manage concurrent tasks. Key tasks include:

*   `host_comm_task`: Handles serial communication with the Klipper host.
*   `motion_control_task`: Manages the stepper motor queue and pulse generation.
*   `thermal_control_task`: Runs the PID loop for all heaters.
*   `safety_monitor_task`: Performs periodic safety checks.

These tasks communicate with each other using message passing (channels) and shared state (mutexes) provided by the `embassy-sync` library. This ensures that access to shared resources (like hardware peripherals) is safe and free from data races.
