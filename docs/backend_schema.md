# r_klipp Backend & IPC Data Schemas

This document defines the binary message formats, data contracts, autoconfig manifest structures, step segment schemas, thermal state-space matrices, and IPC data structures used in **r_klipp**.

---

## 1. Protocol Frame Layout

All host-to-MCU and MCU-to-host serial communications use binary message frames encoded with the `postcard` codec:

```
+------------------+-------------------+-------------------+-------------------+-------------------+
|  Sync Header     |   Sequence ID     |  Payload Length   |  Binary Payload   |   CRC-16 Checksum |
|  (1 Byte: 0x7E)  |   (1 Byte: u8)    |   (2 Bytes: u16)  |  (Var Length)     |   (2 Bytes: u16)  |
+------------------+-------------------+-------------------+-------------------+-------------------+
```

---

## 2. Autoconfig Protocol Schemas (`klipper-proto::autoconfig`)

### 2.1. `HandshakeManifest`
The self-describing hardware manifest sent by the MCU to the host upon connection enumeration:

```rust
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct HandshakeManifest {
    /// Board name identifier (fixed-size UTF-8 byte array)
    pub board_name: [u8; 32],
    /// Core clock frequency in Hertz
    pub clock_speed_hz: u32,
    /// Hardware timer resolution in clock ticks
    pub step_resolution_ticks: u32,
    /// Vector of available GPIO descriptors
    pub pins: heapless::Vec<PinDescriptor, 64>,
}
```

### 2.2. `PinDescriptor` & `PinCapability`
Defines individual GPIO hardware pins and their available hardware functions:

```rust
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct PinDescriptor {
    pub pin_index: u16,
    pub name: [u8; 8],
    pub capabilities_mask: u16,
    pub capabilities: heapless::Vec<PinCapability, 4>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub enum PinCapability {
    DigitalInput,
    DigitalOutput { max_current_ma: u8 },
    AnalogInput { resolution_bits: u8 },
    PwmOutput { max_freq_hz: u32 },
    StepTimerChannel { timer_id: u8 },
}
```

---

## 3. Motion Control & Step Queue Schemas (`r_klipp_motion`)

### 3.1. `StepSegment`
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

### 3.2. SPSC Queue Structure
The MCU ring buffer managed via `heapless::spsc::Queue`:

```rust
pub struct MotionRingBuffer {
    /// Lock-free single-producer single-consumer queue (size 128 segments)
    pub queue: heapless::spsc::Queue<StepSegment, 128>,
}
```

---

## 4. State-Space MPC Thermal Schemas (`r_klipp_thermal`)

### 4.1. Thermal Model State Vector & Matrices
Defines the state-space Model Predictive Control state variables and system matrices:

$$\mathbf{x}_{k+1} = \mathbf{A} \mathbf{x}_k + \mathbf{B} u_k + \mathbf{L} (y_k - \mathbf{C} \mathbf{x}_k)$$

```rust
/// State vector containing estimated temperatures
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MpcStateVector {
    /// Estimated heater core temperature (°C fixed-point)
    pub t_heater: Fixed16_16,
    /// Estimated sensor temperature (°C fixed-point)
    pub t_sensor: Fixed16_16,
}

/// System matrices for State-Space MPC
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MpcMatrices {
    /// System transition matrix A (2x2)
    pub a: [[Fixed16_16; 2]; 2],
    /// Control input matrix B (2x1)
    pub b: [Fixed16_16; 2],
    /// Observation matrix C (1x2)
    pub c: [Fixed16_16; 2],
    /// Kalman gain vector K (2x1)
    pub k_gain: [Fixed16_16; 2],
}
```

---

## 5. Host API & Macro State Schemas (`r_klipp_api` & `klipper-host`)

### 5.1. Host Command Envelope
Structured enum wrapping commands dispatched from host API / Web UI to printer core:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub enum HostCommand {
    GCodeLine(heapless::String<256>),
    ExecuteMacro { name: heapless::String<64>, params: heapless::Vec<(heapless::String<32>, Fixed16_16), 8> },
    SetTargetTemp { heater_id: u8, target_c: Fixed16_16 },
    HomeAxes { axis_mask: u8 },
    EmergencyStop,
}
```

### 5.2. Telemetry Status Frame
High-frequency status data broadcast to client UIs via WebSockets:

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct TelemetryFrame {
    pub timestamp_us: u64,
    pub tool_position: [Fixed16_16; 4], // X, Y, Z, E
    pub current_temps: [Fixed16_16; 4],  // Extruder, Bed, Chamber, Ambient
    pub target_temps: [Fixed16_16; 4],
    pub pwm_power: [u8; 4],              // Duty cycle 0-255
    pub queue_fill_pct: u8,              // Motion buffer saturation
    pub system_state: SystemState,
}
```
