# Klipper MCU Firmware

This crate contains the main `no_std` firmware that runs on the 3D printer's MCU. It is responsible for handling all real-time operations, such as stepper motor control, temperature sensing, and heater management.

## Features

- **Real-Time Control**: Manages all time-sensitive tasks required for 3D printing.
- **Async/Await with Embassy**: Uses the `embassy` framework for cooperative multitasking.
- **RTIC Support (Optional)**: Can be compiled with the `rtic` real-time framework as an alternative scheduler.
- **Safety-Critical**: Includes safety features like watchdog timers and thermal runaway protection.

## Building and Flashing

To build and flash the firmware for a specific board, you can use the following commands.

### Build

```bash
# Build for the mks_skipr board
cargo build --release -p klipper-mcu-firmware --features mks_skipr
```

### Flash

```bash
# Flash the firmware to the mks_skipr board
cargo embed --release -p klipper-mcu-firmware --features mks_skipr
```

## Heater Safety Notice

**WARNING**: The heater control code includes safety interlocks, but testing it carries risks. Improper configuration or bugs could lead to overheating and a fire hazard. Please refer to the [main `README.md`](../../README.md) for detailed safety procedures.

## Contributing

Contributions to improve the firmware are welcome. Please see the main project's [contributor guide](../../docs/contributors.md) for more information.

## License

This crate is licensed under the MIT License. See the [LICENSE](../../LICENSE) file for details.