# Klipper Protocol Crate

This crate provides a canonical Rust implementation of the Klipper host-to-MCU messaging protocol. It is designed for both `std` and `no_std` environments, making it suitable for host-side applications and MCU firmware alike.

## Features

- **Canonical Protocol Implementation**: Correctly implements Klipper's binary framing, compression, and command/response system.
- **`std` and `no_std` Support**: Usable on both host-side applications and on the MCU. The `std` feature is enabled by default.
- **Type-Safe**: Uses Rust's type system to represent Klipper commands and responses, reducing runtime errors.
- **Minimal Dependencies**: Keeps `no_std` dependencies to a minimum.

## Usage

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
klipper-proto = { version = "0.1.0", default-features = false, features = ["alloc"] }
```

For `no_std` environments, disable the default `std` feature and enable `alloc`. For host-side applications, you can enable the `std` feature for additional helpers.

## Protocol Details

The Klipper protocol is a binary protocol that uses a framing mechanism with sync bytes (`0x7E`), a length, a payload, and a CRC checksum. It includes a simple dictionary-based compression scheme. This crate tracks the Klipper protocol as it evolves and provides a `CommandRegistry` to manage the command dictionary established at connection time.

Payloads are serialized using `postcard`, a `serde` format designed for constrained environments.

## Contributing

Contributions to improve the protocol implementation are welcome. Please see the main project's [contributor guide](../../docs/contributors.md) for more information.

## License

This crate is licensed under the MIT License. See the [LICENSE](../../LICENSE) file for details.