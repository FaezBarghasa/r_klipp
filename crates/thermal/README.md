# Klipper in Rust: Thermal Management Crate

## Overview

The `thermal` crate is responsible for all thermal management tasks in the Klipper in Rust firmware. This includes reading temperature sensors, controlling heaters, and implementing the safety features that prevent overheating.

This is a `no_std` library, designed for reliable execution in an embedded environment.

## Features

*   **PID Control**: Implements a Proportional-Integral-Derivative (PID) control loop for precise temperature regulation of heaters.
*   **Sensor Integration**: Interfaces with the `mcu-drivers` crate to read temperature data from various types of sensors, such as thermistors and thermocouples.
*   **Heater Management**: Controls the PWM output to the heaters to maintain the target temperature.
*   **Thermal Safety**: Includes critical safety features, such as:
    *   **Thermal Runaway Protection**: Detects if a heater is not heating up as expected and shuts it down.
    *   **Max Temp Protection**: Prevents the temperature from exceeding a configurable maximum limit.
    *   **Sensor Fault Detection**: Detects if a temperature sensor is disconnected or shorted.
*   **Async Design**: The thermal control loop is implemented as an `embassy` async task, allowing it to run concurrently with other tasks without blocking the system.

## Usage

The `thermal` crate is used by the main `klipper-mcu-firmware` crate. The main application creates a `ThermalManager` task and provides it with the necessary sensor and heater drivers. The `ThermalManager` then takes care of all the details of temperature control and safety.

```rust
// Example of setting up the thermal manager
use thermal::ThermalManager;
use mcu_drivers::driver_thermistor::Thermistor;
use mcu_drivers::heater::Heater;

fn setup_thermal_manager(thermistor: Thermistor, heater: Heater) {
    let mut manager = ThermalManager::new();
    let hotend_id = manager.add_heater(heater, thermistor);
    manager.set_target_temp(hotend_id, 200.0);

    // Spawn the manager task in the embassy executor
    embassy::spawn(manager.run());
}
```
