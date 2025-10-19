# MCU Drivers Crate

This crate provides a collection of `no_std` drivers for various micro-controller peripherals and external components. These drivers are designed to be portable across different hardware platforms by depending only on the `embedded-hal` traits.

## Guiding Principles

- **`no_std` First**: All drivers are designed for bare-metal environments. An `std` feature is provided for host-based testing and simulation.
- **HAL Agnostic**: By relying on `embedded-hal` traits, these drivers can be used with any MCU that has a corresponding HAL implementation.
- **Testable**: Each driver includes a robust test suite using `embedded-hal-mock` to verify correctness without needing physical hardware.
- **Usable by Klipper**: The APIs are designed to be easily integrated into firmware projects like Klipper.

## Available Drivers

- **TMC Drivers**: A driver for Trinamic stepper motor controllers (e.g., TMC2209) supporting configuration and control over UART.
- **Thermistor Drivers**: A library for converting NTC thermistor ADC readings to temperature using either lookup tables or polynomial equations.

## Usage

This crate is a core component of the `klipper-mcu-firmware` and is not intended to be used as a standalone library. The drivers are integrated into the main firmware to control the printer's hardware.

## Contributing

Contributions to add new drivers or improve existing ones are welcome. Please see the main project's [contributor guide](../../docs/contributors.md) for more information.

## License

This crate is licensed under the MIT License. See the [LICENSE](../../LICENSE) file for details.