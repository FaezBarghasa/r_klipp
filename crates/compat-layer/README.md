# Compatibility Layer Crate

This crate provides a compatibility layer for interfacing with C-based components of the Klipper ecosystem. It is intended to bridge the gap between the pure Rust firmware and any external C libraries or legacy code that may be required for certain features.

## Features

- **FFI Bindings**: Contains Foreign Function Interface (FFI) bindings to C-based Klipper code.
- **Data Structure Conversion**: Provides utilities for converting data structures between Rust and C.
- **Safe Abstractions**: Offers safe Rust wrappers around unsafe C functions.

## Usage

This crate is a utility for the main `klipper-mcu-firmware` and is used when interacting with C code. It is not intended for standalone use.

## Contributing

Contributions to improve the compatibility layer are welcome. Please see the main project's [contributor guide](../../docs/contributors.md) for more information.

## License

This crate is licensed under the MIT License. See the [LICENSE](../../LICENSE) file for details.