# `r_klipp` Backend Database & IPC Data Schemas

This document defines the database schemas, wire frames, autoconfig manifests, motion step segments, and REST/WebSocket contracts across the `r_klipp` ecosystem.

---

## 🗄️ 1. SurrealDB Database Tables (`host-server`)

The `host-server` uses embedded **SurrealDB** for time-series telemetry logging, G-Code file metadata, and machine profile persistence.

### 1.1 `gcode_file` Table
```sql
DEFINE TABLE gcode_file SCHEMALESS;
DEFINE FIELD name ON TABLE gcode_file TYPE string;
DEFINE FIELD size ON TABLE gcode_file TYPE int;
DEFINE FIELD upload_date ON TABLE gcode_file TYPE string;
DEFINE FIELD metadata ON TABLE gcode_file TYPE object;
```

### 1.2 `print_history` Table
```sql
DEFINE TABLE print_history SCHEMALESS;
DEFINE FIELD filename ON TABLE print_history TYPE string;
DEFINE FIELD start_time ON TABLE print_history TYPE string;
DEFINE FIELD end_time ON TABLE print_history TYPE option<string>;
DEFINE FIELD status ON TABLE print_history TYPE string; -- 'Printing', 'Completed', 'Cancelled', 'Failed'
DEFINE FIELD telemetry_summary ON TABLE print_history TYPE object;
```

### 1.3 `machine_config` Table
```sql
DEFINE TABLE machine_config SCHEMALESS;
DEFINE FIELD machine_type ON TABLE machine_config TYPE string; -- 'ThreeDPrinter', 'PnpPip', 'Cnc'
DEFINE FIELD kinematics ON TABLE machine_config TYPE string;
DEFINE FIELD axis_limits ON TABLE machine_config TYPE array;
```

---

## 🛰️ 2. Serial & Network Frame Layout

All host-to-MCU communications use binary message frames encoded with COBS and Postcard:

```
+------------------+-------------------+-------------------+-------------------+
|  COBS Stuffed    |   Message Type    |  Binary Payload   |   CRC-16 CCITT    |  0x00 Delimiter
|  (Variable Len)  |   (Enum Discriminant) | (Postcard format) |   (2 Bytes: u16)  |  (1 Byte)
+------------------+-------------------+-------------------+-------------------+
```

---

## ⚡ 3. Motion Control Schemas (`motion` & `kinematics`)

### 3.1 `StepSegment`
The fundamental unit of motion queued in the SPSC ring buffer for DMA stepper pulse generation:

```rust
#[repr(C)]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct StepSegment {
    /// Initial interval between step pulses in MCU clock ticks
    pub interval_ticks: u32,
    /// Acceleration rate represented as tick decrement per step
    pub tick_change: i32,
    /// Total number of step pulses in this motion segment
    pub step_count: u32,
    /// Bitmask for motor direction pins (1 bit per axis/stepper)
    pub dir_mask: u32,
    /// Bitmask for motor enable pins
    pub enable_mask: u32,
}
```

### 3.2 `SyncIoAction` (Pick & Place Discrete I/O)
```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct SyncIoAction {
    pub pin_id: u8,
    pub target_state: bool,
    pub pulse_duration_us: u32,
    pub trigger_timestamp_us: u64,
}
```

---

## 🌡️ 4. State-Space MPC Thermal Schemas (`thermal`)

$$\mathbf{x}_{k+1} = \mathbf{A} \mathbf{x}_k + \mathbf{B} u_k + \mathbf{K} (y_k - \mathbf{C} \mathbf{x}_k)$$

```rust
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MpcStateVector {
    pub t_heater: f32,
    pub t_sensor: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpcMatrices {
    pub a: [[f32; 2]; 2],
    pub b: [f32; 2],
    pub c: [f32; 2],
    pub k_gain: [f32; 2],
}
```
