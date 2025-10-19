# Safety Subsystem

The safety of the user and the machine is the highest priority in the Klipper in Rust firmware. This document details the multi-layered safety subsystem designed to prevent accidents, detect faults, and ensure the printer always operates in a known, safe state.

## 1. Guiding Principles

*   **Fail-Safe Design**: In the event of a fault, the firmware will always transition the hardware to a safe state (e.g., heaters off, motors disabled).
*   **Redundancy**: Critical safety checks are performed by multiple components to provide redundancy.
*   **Hardware and Software Interlocks**: Safety is enforced by a combination of hardware features (e.g., watchdog timers) and software logic.
*   **User Notification**: When a safety event occurs, the firmware will make a best effort to inform the user via the Klipper host.

## 2. Key Safety Features

### 2.1. Watchdog Timers

*   **Hardware Watchdog (IWDG)**: The MCU's independent watchdog is used to guard against firmware lockups. The main loop must "pet" the watchdog at regular intervals. If it fails to do so (e.g., due to an infinite loop), the watchdog will reset the MCU, bringing it back to a known state.
*   **Software Watchdogs**: Individual tasks (e.g., the thermal control loop) have their own software watchdogs. If a task fails to check in on time, the safety monitor will trigger an emergency stop.

### 2.2. Thermal Safety

Overheating is a significant fire hazard. The following mechanisms are in place to prevent it:

*   **Thermal Runaway Protection**:
    *   The firmware monitors the rate of temperature increase when a heater is turned on. If the temperature does not rise by a minimum amount within a set time, it indicates a fault (e.g., a detached heater cartridge), and the heater is shut down.
    *   It also checks if the temperature exceeds a maximum configurable limit (`max_temp`).
*   **Thermistor Fault Detection**:
    *   **Open Circuit**: If a thermistor is disconnected, its resistance will go to infinity, resulting in a very low ADC reading. The firmware detects this and disables the corresponding heater.
    *   **Short Circuit**: If a thermistor's wires are shorted, its resistance will drop to near zero, causing a very high ADC reading. This condition also triggers a heater shutdown.
*   **PID Sanity Checks**: The PID controller's output is clamped to prevent it from demanding 100% power for an extended period if the target temperature is not being reached.

### 2.3. Motion Safety

*   **Endstop Monitoring**: The firmware continuously monitors the state of the endstops. If an endstop is triggered unexpectedly, it can be configured to halt motion.
*   **Stepper Driver Fault Detection**: For smart stepper drivers (e.g., TMC2209), the firmware can monitor their fault pins. If a driver reports an error (e.g., over-temperature or short-circuit), the motors are disabled.
*   **Maximum Velocity and Acceleration**: The host is responsible for commanding valid motion profiles, but the firmware includes sanity checks to reject commands that would exceed the machine's physical limits.

## 3. Emergency Stop (E-Stop)

An emergency stop is the ultimate safety mechanism. It can be triggered by:

*   A command from the Klipper host (`M112`).
*   The safety monitor detecting a critical fault.
*   A physical E-stop button (if configured).

When an E-stop is triggered, the firmware performs the following actions in a hard real-time context:

1.  **Immediately disable all heaters** by turning off their PWM outputs and disabling their GPIO pins.
2.  **Immediately disable all stepper motors** by deactivating their `enable` pins.
3.  **Halt all motion planning** and clear the motion queue.
4.  Send a `Shutdown` message to the host.
5.  Enter an infinite loop, requiring a hardware reset to recover.

## 4. Implementation Details

*   **Safety Monitor Task**: A dedicated, high-priority async task is responsible for executing periodic safety checks.
*   **Atomic State**: A global atomic `SystemState` variable is used to track the overall health of the system. In the case of a fault, this state is atomically switched to `Fault`, and all other tasks are required to check this state and act accordingly.
*   **Hardware Abstraction**: The safety features are designed to be portable across different MCUs by leveraging the traits provided by the `embedded-hal`.

By combining these hardware and software features, the Klipper in Rust firmware aims to provide a robust and reliable platform for 3D printing, with safety as its core foundation.
