# Klipper Protocol Overview

This document describes the communication protocol between the Klipper host and the Klipper in Rust MCU firmware. The protocol is designed to be efficient, low-latency, and robust.

---

## 1. Protocol Basics

- **Transport Layer**: The protocol is typically transmitted over a serial line (USB CDC-ACM or UART).
- **Message Framing**: The protocol uses a simple framing mechanism to delineate messages. Each message is prefixed with a sync byte (`0x7E`) and followed by a checksum.
- **Message Structure**: Messages are binary-encoded and consist of a message ID and a payload. The `klipper-proto` crate contains the definitions for all message types and their parameters.

---

## 2. Message Flow

The communication is bidirectional, with both the host and the MCU sending messages.

- **Host to MCU**: The host sends commands to the MCU to control the printer's hardware. Examples include:
  - `Queue a motion block` (enqueueing `StepSegment` data)
  - `Set heater temperature` (updating targets in the MPC thermal engine)
  - `Read ADC value`
  - `Configure GPIO pin`
- **MCU to Host**: The MCU sends responses and status updates to the host. Examples include:
  - `Command acknowledged`
  - `Temperature reading`
  - `Endstop triggered`
  - `Error condition detected` / `Shutdown`

---

## 3. Key Message Types

### Command Messages (Host -> MCU)

These messages instruct the MCU to perform an action.

- `Config_stepper`: Configures a stepper motor with its associated pins and parameters.
- `Queue_step`: Adds a step segment (a series of timed step pulses containing `interval_ticks`, direction, and enable masks) to the SPSC motion queue.
- `Set_heater_temperature`: Sets the target temperature for the Model Predictive Control (MPC) engine.
- `Emergency_stop`: Commands the MCU to immediately enter a safe state.

### Response Messages (MCU -> Host)

These messages provide feedback and data from the MCU.

- `Klipper_ready`: Sent by the MCU on startup to signal that it is ready to receive commands.
- `Steptrigger`: Reports that a stepper motor has completed a step.
- `Adc_state`: Periodically sends the latest ADC readings for thermistors.
- `Shutdown`: Sent by the MCU when it is entering a shutdown state due to an error.

---

## 4. Self-Describing Pinout Configuration Protocol (Autoconfig)

To decouple host configurations from static firmware pinouts, the system utilizes a **Self-Describing Pinout Configuration Protocol** located in [autoconfig.rs](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/crates/klipper-proto/src/autoconfig.rs).

### 4.1. Manifest structures

During startup enumeration, the MCU compiles a `HandshakeManifest` detailing its physical resources:

```rust
pub struct HandshakeManifest {
    pub board_name: [u8; 32],
    pub clock_speed_hz: u32,
    pub step_resolution_ticks: u32,
    pub pins: heapless::Vec<PinDescriptor, 64>,
}
```

Each pin is represented by a `PinDescriptor` describing its indexing, UTF-8 fixed-length name, capabilities mask, and capabilities vectors:

```rust
pub struct PinDescriptor {
    pub pin_index: u16,
    pub name: [u8; 8],
    pub capabilities_mask: u16,
    pub capabilities: heapless::Vec<PinCapability, 4>,
}
```

The available capabilities are defined in the `PinCapability` enum:

- `DigitalInput`
- `DigitalOutput { max_current_ma: u8 }`
- `AnalogInput { resolution_bits: u8 }`
- `PwmOutput { max_freq_hz: u32 }`
- `StepTimerChannel { timer_id: u8 }`

### 4.2. Handshake Timing & Serialization

1. **Protocol Negotiation**: The manifest negotiation must succeed within **1.5 seconds** of serial port enumeration.
2. **Postcard Serialization**: Because the MCU operates under `no_std`, serialization is performed using the `postcard` binary codec.
3. **Zero-Copy Deserialization**: The host deserializes capabilities directly out of the receive buffers without dynamic memory allocations, optimizing startup speeds and memory usage.

---

## 5. Message Encoding and Decoding

The `klipper-proto` crate provides the serialization and deserialization logic for all messages.

### Serialization (Host Side)
1. A command is constructed in the host software (e.g., Klippy).
2. The command's parameters are packed into a binary format according to the message definition.
3. A checksum is calculated over the message payload.
4. The final message, including the sync byte, payload, and checksum, is sent over the serial link.

### Deserialization (MCU Side)
1. The MCU's serial task continuously reads from the serial port, looking for the `0x7E` sync byte.
2. Once a sync byte is found, the MCU reads the message length and payload.
3. The checksum is verified to ensure data integrity.
4. If the checksum is valid, the message payload is passed to the command dispatcher, which decodes the message ID and parameters.
5. The corresponding command handler is executed.
