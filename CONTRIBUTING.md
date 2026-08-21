# Contributing to r_klipp

Thank you for your interest in contributing to `r_klipp`! `r_klipp` is a next-generation, high-performance, deterministic motion control firmware and host ecosystem built in Rust.

## Principles & Code Standards

1. **Zero-Allocation in Core Crates**: The core crates (`parser`, `motion`, `thermal`, `safety`, `kinematics`, `comms`, `klipper-proto`, `hal`, `mcu-drivers`) must maintain `#![no_std]` compatibility and must never allocate on the heap in real-time execution loops. Use `heapless`, stack buffers, or pre-allocated structures.
2. **Deterministic Step Generation**: All motion calculations and step pulse intervals must be jitter-free. The CPU feeds DMA buffers; timing is governed strictly by hardware timers.
3. **Fail-Safe Safety First**: Watchdog supervisors and thermal runaway checks are mandatory. Any violation must immediately cut power to all heaters and disable steppers.
4. **Clean Dependencies**: Never add heavy host-side dependencies or GUI/audio libraries into embedded or simulator core crates.

## Development Workflow

### 1. Formatting and Linting
Before submitting a PR, ensure all formatting, clippy lints, and cargo deny checks pass:
```bash
cargo fmt --all -- --check
cargo clippy --workspace --exclude klipper-mcu-firmware --exclude r_klipp_firmware -- -D warnings
cargo deny check
```

### 2. Running Core Tests
Run all unit and integration tests across the workspace:
```bash
cargo test -p parser -p motion -p thermal -p safety -p kinematics -p comms -p klipper-proto -p sim -p compat-layer
```

### 3. Property-Based Testing
Verify motion and parser invariants under random workloads:
```bash
cargo test --test proptest_motion
cargo test --test proptest_parser
```

## Pull Request Guidelines
- Ensure all CI checks pass.
- Include unit tests or property-based tests for new math or parsing logic.
- Keep commits atomic and clearly described.
