# Klipper in Rust: Protocol Definition Crate

## Overview

The `klipper-proto` crate provides the core data structures and serialization/deserialization logic for the Klipper communication protocol. It is a foundational crate used by both the MCU firmware and host-side tools to ensure that both ends of the communication link are in sync.

This crate is a `no_std` library, making it suitable for use in resource-constrained embedded environments.

## Features

*   **Strongly-Typed Messages**: All Klipper protocol messages are defined as Rust enums and structs, leveraging the type system to prevent errors related to incorrect message formats or parameter types.
*   **Zero-Copy Deserialization**: The deserialization logic is designed to be highly efficient, avoiding memory allocations where possible by parsing messages directly from the receive buffer.
*   **Compile-Time Correctness**: By defining the protocol in a single, shared crate, we can verify at compile time that the firmware and host tools are compatible.
*   **Extensibility**: The protocol definition is designed to be easily extended with new messages and parameters as the firmware evolves.

## Usage

This crate is primarily used as a dependency by other crates in the workspace.

### Defining a Message

Messages are defined in the `src/messages.rs` file. Each message is a struct that derives the `Serialize` and `Deserialize` traits.

```rust
// Example of a message definition
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SetHeaterTemperature {
    pub heater_id: u8,
    pub target_temp: f32,
}
```

### Serializing and Deserializing

The crate provides functions for serializing a message struct into a byte buffer and for deserializing a byte buffer back into a message struct.

```rust
use klipper_proto::messages::{Message, SetHeaterTemperature};
use klipper_proto::codec::KlipperCodec;

fn example_usage() {
    let message = Message::SetHeaterTemperature(SetHeaterTemperature {
        heater_id: 0,
        target_temp: 200.0,
    });

    // Serialize the message
    let mut buffer = [0u8; 64];
    let encoded_len = KlipperCodec::encode(&message, &mut buffer).unwrap();

    // Deserialize the message
    let decoded_message = KlipperCodec::decode(&buffer[..encoded_len]).unwrap();

    assert_eq!(message, decoded_message);
}
```
