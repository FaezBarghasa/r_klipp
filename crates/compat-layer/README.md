# Klipper in Rust: Compatibility Layer Crate

## Overview

The `compat-layer` crate provides a compatibility layer to interface with existing C-based Klipper components or other C libraries. This is crucial for incremental migration and for leveraging battle-tested C codebases where a pure-Rust alternative is not yet available.

This crate is designed to be a temporary bridge, with the long-term goal of replacing all C dependencies with safe, idiomatic Rust code.

## Features

*   **C-ABI Bindings**: Provides safe Rust wrappers around C functions and data structures.
*   **Memory Management**: Includes utilities for safely managing memory that is shared between Rust and C.
*   **Error Handling**: Defines a clear strategy for propagating errors between the two language boundaries.

## Usage

To use this crate, add it as a dependency in your `Cargo.toml` and use the provided wrapper functions to call into the C code.

```rust
// Example of calling a C function via the compat layer
use compat_layer::legacy_c_code;

fn do_something_with_c() {
    // The unsafe block is necessary when calling C functions
    unsafe {
        legacy_c_code::do_something();
    }
}
```

## Safety

**Warning**: This crate contains a significant amount of `unsafe` code by its very nature. Interfacing with C requires careful handling of raw pointers, memory allocation, and data lifetimes. All `unsafe` blocks are carefully reviewed and documented, but they remain a potential source of bugs.

When using this crate, it is the caller's responsibility to uphold the safety invariants required by the underlying C functions.
