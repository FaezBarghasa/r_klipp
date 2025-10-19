# Developer Workflow

This document outlines the recommended workflow for contributing to the Klipper Rust project. Following these guidelines will help ensure that your contributions are easy to review and integrate.

## Development Environment

To get started, you will need to set up your development environment. Please see the "Getting Started" section in the main [`README.md`](../README.md) for instructions on how to install the Rust toolchain and other dependencies.

## Building and Testing

Before submitting any changes, it is important to ensure that the code builds and that all tests pass.

### Building the Firmware

To build the firmware, you can use the following command:

```bash
cargo build --all-targets
```

This will build all the crates in the workspace, including the firmware and the host software.

### Running Tests

To run the test suite, you can use the following command:

```bash
cargo test --all
```

This will run all the unit tests and integration tests for all the crates in the workspace.

### Running in the Simulator

The simulator allows you to run the firmware on your host machine, which is useful for testing and debugging without physical hardware. To run the simulator, use the following command:

```bash
cargo run -p klipper-mcu-firmware --features sim
```

## Submitting Changes

Once you have made your changes and verified that everything is working correctly, you can submit a pull request.

### Code Style

Please ensure that your code is formatted using `rustfmt`. You can run `rustfmt` on your code with the following command:

```bash
cargo fmt --all
```

### Pull Requests

When creating a pull request, please provide a clear and descriptive title and a detailed description of the changes you have made. If your changes address an existing issue, please reference the issue number in the pull request description.

## Contributor Guide

For more detailed information on contributing to the project, please see the [contributor guide](./contributors.md).