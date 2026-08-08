# r_klipp: Advanced Klipper Firmware in Rust

**r_klipp** is a master workspace for a cutting-edge, real-time control system and safety engine, written from the ground up in Rust. It offers a `no_std`, bare-metal, and memory-safe alternative to traditional Klipper firmware, designed for high-performance 3D printers and CNC machines.

The system is architected around a dual-paradigm concurrency model, leveraging both cooperative multitasking for non-blocking I/O and preemptive real-time interrupts for time-critical operations. This ensures deterministic, jitter-free performance, even at high step rates.

## Key Features

- **Bare-Metal Safety**: Built on a `no_std` foundation, `r_klipp` enforces memory safety at compile time, eliminating entire classes of runtime errors.
- **Advanced Motion Planning**: Incorporates a Pythagorean-Hodograph (PH) corner blending and a G4 31-phase trajectory generator for smooth, high-speed motion.
- **State-Space MPC Thermal Control**: Utilizes a state-space Model Predictive Control (MPC) engine with Kalman filtering for precise and stable temperature management.
- **Multi-MCU Clock Synchronization**: A Distributed Phase-Locked Loop (DPLL) ensures that multiple MCUs can coordinate stepper timing with sub-microsecond accuracy.
- **Host-MCU Autoconfig Protocol**: A self-describing hardware manifest and `postcard` serialization enable plug-and-play MCU configuration.

For a deep dive into the system's design, see the [Architecture Document](docs/architecture.md).

## Project Status

This project is under active development. For recent changes, see the [Changelog](docs/changelog.md).

## Contributing

We welcome contributions! Please see our [Contribution Guide](docs/contributors.md) and [Code of Conduct](docs/CODE_OF_CONDUCT.md) for more information.
