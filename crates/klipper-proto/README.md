# `klipper-proto`: Wire Protocols, DPLL Clock Sync & Binary Codecs

`klipper-proto` is a foundational, `no_std` wire protocol and serialization library providing binary message codecs, COBS framing, CRC-16 checksums, DPLL clock synchronization, and CAN-FD peripheral RPC schemas.

---

## 📦 Core Modules

### 1. `clock_sync.rs` (Distributed Phase-Locked Loop)
- Implements `DpllClockSynchronizer` to align multi-MCU hardware timers with sub-microsecond precision over serial and CAN buses.

### 2. `codec.rs` & `crc.rs`
- **COBS Framing**: Zero-byte boundary framing with minimal overhead.
- **CRC-16 CCITT**: $0x1021$ polynomial verification on every incoming packet.
- **Postcard Codec**: Zero-allocation binary encoding/decoding for embedded Cortex-M targets.

### 3. `feeder.rs` (Pick & Place CAN-FD Feeder Protocol)
- Strongly typed schema for smart component tape feeders (`Advance`, `Peel`, `CalibratePitch`, and telemetry queries).

### 4. `autoconfig.rs`
- Handshake manifest negotiation providing plug-and-play board identification and GPIO capability introspection.

---

## 🧪 Testing
```bash
cargo test -p klipper-proto
```
