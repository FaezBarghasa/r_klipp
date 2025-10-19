# Klipper Host Crate

This crate contains the host-side software for the Klipper Rust project. It is a high-performance, safe, and concurrent implementation of the Klipper host process, written in Rust. It is intended to be a drop-in replacement for the Python-based `klippy`.

## Features

- **Performance**: Built with Rust and Tokio for high-throughput, low-latency processing of G-code.
- **Safety**: Leverages Rust's compile-time safety checks to prevent common bugs.
- **Compatibility**:
  - Parses standard `printer.cfg` files.
  - Provides a `/tmp/printer` virtual serial port for compatibility with existing software like OctoPrint.
  - Offers a REST and WebSocket API compatible with modern web UIs like Fluidd and Mainsail.
- **Modularity**: Clear separation of concerns between components like G-code processing, motion planning, MCU communication, and the web API.

## Getting Started

To run the host process, you will need a `printer.cfg` file.

```bash
cargo run -p klipper-host -- --config-path /path/to/your/printer.cfg
```

For testing without a physical printer, you can use the mock MCU:

```bash
cargo run -p klipper-host -- --mock-mcu
```

## Contributing

Contributions to improve the host software are welcome. Please see the main project's [contributor guide](../../docs/contributors.md) for more information.

## License

This project is licensed under the GNU General Public License v3.0, in accordance with the original Klipper project. See the `LICENSE` file for details.