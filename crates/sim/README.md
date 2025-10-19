# Klipper Firmware Simulator

This crate provides a simulation environment for the Klipper Rust firmware. It allows the firmware to be compiled and run on a host machine, which is useful for development, testing, and debugging without requiring physical hardware.

## Features

- **Hardware Abstraction**: Simulates the MCU hardware, allowing the firmware to run on a standard computer.
- **Testing and Debugging**: Enables rapid testing of firmware logic in a controlled environment.
- **CI Integration**: Can be used in Continuous Integration (CI) pipelines to run automated tests.

## Usage

This crate is a development tool and is not intended for production use. To run the simulator, you can use the following command:

```bash
cargo run -p klipper-mcu-firmware --features sim
```

This will build and run the firmware in simulation mode.

## Contributing

Contributions to improve the simulator are welcome. Please see the main project's [contributor guide](../../docs/contributors.md) for more information.

## License

This crate is licensed under the MIT License. See the [LICENSE](../../LICENSE) file for details.