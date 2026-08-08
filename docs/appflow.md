# r_klipp System Application Flow & Operations

This document details the end-to-end operational workflows, system state transitions, command execution pipelines, user interface flows, and error recovery sequences in **r_klipp**.

---

## 1. System High-Level Lifecycle Flow

```mermaid
stateDiagram-v2
    [*] --> PowerOn: Hardware Initialization
    PowerOn --> FirmwareBoot: RTIC 2 / Embassy Executor Init
    FirmwareBoot --> AutoconfigHandshake: Serial / USB CDC-ACM Connect
    AutoconfigHandshake --> DPLLSynchronization: Manifest Exchanged & Validated
    DPLLSynchronization --> IdleReady: Sub-microsecond Clock Locked
    
    state IdleReady {
        [*] --> WaitingForCommands
        WaitingForCommands --> ParsingGCode: Receive G-Code / API Request
        ParsingGCode --> PlanningTrajectory: Rhai Macro Expansion & Parsing
        PlanningTrajectory --> StreamingStepSegments: PH Bezier & G4 Trajectory Planning
    }

    IdleReady --> ActivePrinting: Execute Motion Queue
    
    state ActivePrinting {
        [*] --> StepPulseGeneration
        StepPulseGeneration --> MPCThermalControl: DMA Double Buffer Stepping
        MPCThermalControl --> SafetyMonitoring: State-Space Temperature Loop
    }
    
    ActivePrinting --> IdleReady: Job Complete / Stopped
    ActivePrinting --> EmergencyShutdown: Hardware Exception / Safety Trip
    IdleReady --> EmergencyShutdown: E-Stop Button / Thermal Runaway
    
    state EmergencyShutdown {
        [*] --> CutPowerPins
        CutPowerPins --> ClearStepQueues
        ClearStepQueues --> SignalHostError
    }
    
    EmergencyShutdown --> [*]: Hard Reset Required
```

---

## 2. Startup & Handshake Protocol Flow

1. **Firmware Initialization**:
   - MCU powers on; RTIC 2 priority interrupt hardware and Embassy async executor initialize.
   - Peripherals (GPIO, Timers, DMA, ADCs) are configured into a known safe state (all heaters disabled).
2. **Self-Describing Autoconfig Manifest Transmission**:
   - The MCU constructs a `HandshakeManifest` listing all board pins, clock speeds, and capability vectors (`PinCapability`).
   - The manifest is serialized using the `postcard` binary protocol and sent over USB/UART with sync header (`0x7E`).
3. **Host Validation & Protocol Lock**:
   - `klipper-host` deserializes `HandshakeManifest` without dynamic allocation.
   - The host verifies hardware parameters against the printer configuration file within **1.5 seconds**.
4. **DPLL Clock Synchronization Initialization**:
   - Host sends continuous time-sync ping packets.
   - MCU captures local hardware timer ticks and returns timestamps.
   - Host computes recursive least squares linear regression ($y = mx + c$) to establish clock alignment across single or multi-MCU setups.

---

## 3. G-Code Processing & Motion Execution Pipeline Flow

```mermaid
sequenceDiagram
    autonumber
    actor User/UI as Host UI / Web API
    participant HostParser as G-Code Parser & Rhai VM
    participant MotionPlanner as PH & G4 Trajectory Planner
    participant ProtoBridge as Postcard Serializer
    participant MCUQueue as MCU SPSC Step Queue
    participant StepperDMA as DMA Step Engine
    participant Hardware as Stepper Motors

    User/UI->>HostParser: Send G-Code / Macro (e.g. G1 X100 Y50 F6000)
    HostParser->>HostParser: Expand Rhai Macros & Check Limits
    HostParser->>MotionPlanner: Pass Continuous Path Target
    MotionPlanner->>MotionPlanner: Apply PH Bezier Corner Blending ($C^4$)
    MotionPlanner->>MotionPlanner: Calculate G4 31-Phase Trajectory Profile
    MotionPlanner->>ProtoBridge: Generate StepSegment Sequence
    ProtoBridge->>MCUQueue: Stream Encoded Packets over Transport (0x7E Frame)
    MCUQueue->>StepperDMA: Push StepSegments to SPSC Ring Buffer
    StepperDMA->>Hardware: Pulse STEP/DIR Pins via Timer DMA (up to 1 MHz)
```

---

## 4. State-Space MPC Thermal Control Flow

```mermaid
flowchart TD
    A[ADC Sensor Reading] --> B[Fixed-Point Thermistor Conversion]
    B --> C[Kalman Filter Prediction Step]
    C --> D[Update State Estimates: T_sensor & T_heater]
    D --> E{Feed-Forward Active?}
    E -- Yes --> F[Add Extrusion Flow Heat Compensation]
    E -- No --> G[Standard MPC State Feedback Calculation]
    F --> H[Compute PWM Duty Cycle]
    G --> H
    H --> I[Output Hardware PWM Signal]
    H --> J[Verify Safety Bounds]
    J -- Temp Out of Bounds --> K[Trigger Thermal Runaway Emergency Shutdown]
    J -- Temp Normal --> L[Continue Thermal Loop]
```

---

## 5. Homing & Calibration Workflow

1. **Braking Curve Safe Homing**:
   - Host issues homing command for axis ($X$, $Y$, or $Z$).
   - Motion planner calculates velocity profile bound by an **85% safety factor braking distance**.
   - Stepper engine moves axis towards endstop/sensor at specified homing speed.
2. **Endstop Trigger & Preemptive Lock**:
   - Hardware interrupt triggers instantly when endstop changes state.
   - DMA step generation for the homing axis is halted in hardware ($< 1 \mu s$).
   - Current motor position is recorded into hardware registers and returned to host.
3. **Axis Backoff & Precision Re-probe**:
   - Axis backs off by pre-configured distance and re-approaches at low velocity for sub-micron zero positioning.

---

## 6. Emergency Stop & Fault Recovery Sequence

When a critical fault occurs (e.g., thermal runaway, loss of DPLL sync, watchdog timeout, physical E-Stop):

1. **MCU Hard-Stop Interrupt**:
   - RTIC highest-priority interrupt cuts power to all heater PWM pins immediately.
   - Stepper EN (Enable) pins are pulled HIGH/LOW to disable motor torque.
2. **Buffer Flush & Host Notification**:
   - MCU flushes SPSC step queues and cancels active DMA buffers.
   - MCU emits `Shutdown` packet (`0x7E` header) with precise error code (e.g., `ERR_THERMAL_RUNAWAY`, `ERR_WATCHDOG_TIMEOUT`).
3. **Host State Lockdown**:
   - `klipper-host` transitions to `EmergencyShutdown` state.
   - All client UIs display emergency alert modal with fault diagnostic trace.
   - System requires manual user clearing (`M112` acknowledge & host restart) before power can be restored to actuators.
