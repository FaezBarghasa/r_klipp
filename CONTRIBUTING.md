# Contributing to `r_klipp`

Thank you for your interest in contributing to **`r_klipp`**! We welcome contributions from firmware engineers, motion control researchers, embedded Rustaceans, and machine operators across 3D printing, Pick & Place (PnP), and CNC domains.

---

## 🏛️ Core Architectural Principles

1. **`no_std` Zero-Allocation Core**:
   - The core crates (`hal`, `parser`, `motion`, `thermal`, `safety`, `kinematics`, `comms`, `klipper-proto`, `mcu-drivers`) must compile with `#![no_std]` and `#![forbid(unsafe_code)]` wherever possible.
   - Never perform dynamic heap allocations (`alloc`, `Box`, `Vec`) inside high-frequency planning or interrupt loops. Utilize `heapless`, fixed-size buffers, or pre-allocated rings.
2. **Deterministic Step Generation**:
   - Step pulse trains and trajectory planning must produce bit-identical, jitter-free timing across target platforms.
3. **Fail-Safe Safety Boundaries**:
   - Hardware watchdogs, touch probe supervisors, and thermal runaway guards must never be disabled or bypassed.
4. **Clean Crate Isolation**:
   - Keep host-only frameworks (Actix, SurrealDB, Slint) strictly inside `host-server` and `host-ui`.

---

## 🛠️ Developer Checklist Before Submitting a PR

### 1. Code Formatting & Linting
Ensure all code conforms to workspace style and strict warnings:
```bash
# Check code formatting
cargo fmt --all -- --check

# Run strict Clippy linter across host and core crates
cargo lint

# Run Clippy on embedded ARM targets
cargo lint-mcu
```

### 2. Run the Full Test Suite
All unit, integration, and property tests must pass:
```bash
cargo test -p parser -p motion -p kinematics -p thermal -p safety -p sim -p klipper-proto -p compat-layer -p r_klipp_api -p host-ui -p host-server
```

### 3. Property-Based & Determinism Tests
```bash
cargo test --test proptest_motion -- --nocapture
cargo test --test proptest_parser -- --nocapture
cargo test --test determinism_test -- --nocapture
```

---

## 📬 Pull Request Process

1. Fork the repository and create a feature branch (`feature/your-feature-name` or `fix/issue-description`).
2. Include comprehensive unit tests and documentation comments for all public traits and structs.
3. Update relevant documentation in `docs/` and crate `README.md` if public API or configuration schemas change.
4. Ensure all CI workflows pass before requesting review.
