# Klipper Protocol Overview

This document describes the communication protocol between the Klipper host and the Klipper in Rust MCU firmware. The protocol is designed to be efficient, low-latency, and robust.

## 1. Protocol Basics

*   **Transport Layer**: The protocol is typically transmitted over a serial line (USB CDC-ACM or UART).
*   **Message Framing**: The protocol uses a simple framing mechanism to delineate messages. Each message is prefixed with a sync byte (`0x7E`) and followed by a checksum.
*   **Message Structure**: Messages are binary-encoded and consist of a message ID and a payload. The `klipper-proto` crate contains the definitions for all message types and their parameters.

## 2. Message Flow

The communication is bidirectional, with both the host and the MCU sending messages.

*   **Host to MCU**: The host sends commands to the MCU to control the printer's hardware. Examples include:
    *   `Queue a motion block`
    *   `Set heater temperature`
    *   `Read ADC value`
    *   `Configure GPIO pin`
*   **MCU to Host**: The MCU sends responses and status updates to the host. Examples include:
    *   `Command acknowledged`
    *   `Temperature reading`
    *   `Endstop triggered`
    *   `Error condition detected`

## 3. Key Message Types

### Command Messages (Host -> MCU)

These messages instruct the MCU to perform an action.

*   `Config_stepper`: Configures a stepper motor with its associated pins and parameters.
*   `Queue_step`: Adds a motion block (a series of timed step pulses) to the motion queue.
*   `Set_heater_temperature`: Sets the target temperature for a PID-controlled heater.
*   `Emergency_stop`: Commands the MCU to immediately enter a safe state.

### Response Messages (MCU -> Host)

These messages provide feedback and data from the MCU.

*   `Klipper_ready`: Sent by the MCU on startup to signal that it is ready to receive commands.
*   `Steptrigger`: Reports that a stepper motor has completed a step.
*   `Adc_state`: Periodically sends the latest ADC readings for thermistors.
*   `Shutdown`: Sent by the MCU when it is entering a shutdown state due to an error.

## 4. Message Encoding and Decoding

The `klipper-proto` crate provides the serialization and deserialization logic for all messages.

### Serialization (Host Side)

1.  A command is constructed in the host software (e.g., Klippy).
2.  The command's parameters are packed into a binary format according to the message definition.
3.  A checksum is calculated over the message payload.
4.  The final message, including the sync byte, payload, and checksum, is sent over the serial link.

### Deserialization (MCU Side)

1.  The MCU's serial task continuously reads from the serial port, looking for the `0x7E` sync byte.
2.  Once a sync byte is found, the MCU reads the message length and payload.
3.  The checksum is verified to ensure data integrity.
4.  If the checksum is valid, the message payload is passed to the command dispatcher, which decodes the message ID and parameters.
5.  The corresponding command handler is executed.

## 5. Protocol Extensions in Rust

The Rust implementation of the protocol aims to be fully compatible with the official Klipper protocol, but it also introduces some enhancements for safety and performance:

*   **Strongly Typed Messages**: Using Rust's enums and structs, all message types are defined at compile time. This prevents errors related to incorrect message formatting or parameter types.
*   **Zero-Copy Deserialization**: Where possible, the deserialization process avoids allocating new memory, instead parsing the message directly from the receive buffer. This reduces latency and improves performance.
*   **Compile-Time Verification**: The `klipper-proto` crate can be used by both the MCU firmware and host-side tools, ensuring that both ends of the communication link are always in sync with the protocol definition.
