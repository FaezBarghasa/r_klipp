# System Architecture Diagrams

This document contains Mermaid diagrams illustrating the structure, data flows, execution pipelines, and concurrency models of the `r_klipp` system.

---

## 1. System Topology & Data Flow

This diagram shows the complete path from high-level G-Code command execution down to bare-metal motor stepping and thermal feedback loops.

```mermaid
graph TD
    %% Host components
    subgraph Host ["Klipper Host (std)"]
        Parser["G-Code Parser"]
        MacroEngine["Host Macro Engine (Rhai VM)"]
        Planner["Kinematic Planner (PH & G4)"]
        HIL["HIL & Input Shaper Calibrator"]
        SerDeHost["Postcard Serializer/Deserializer"]
    end

    %% Communication
    SerLink{{"Serial Transport (UART/USB)"}}

    %% MCU components
    subgraph MCU ["Microcontroller Firmware (no_std, RTIC 2 & Embassy)"]
        SerDeMCU["Postcard Parser (Rx/Tx)"]
        Dispatcher["Command Dispatcher"]
        ClockSync["Clock Sync (DPLL)"]
        StepQueue["SPSC Step Queue (StepSegment)"]
        DmaStepper["DmaStepEngine (Double-Buffer)"]
        ThermalMPC["MpcThermalEngine (Kalman MPC)"]
        Safety["SafetyMonitor (Watchdog)"]
    end

    %% Physical Hardware
    subgraph Hardware ["Physical Printer Hardware"]
        Motors["Stepper Motors (X, Y, Z, E)"]
        Heaters["PWM Heaters (Hotend, Bed)"]
        Thermistors["ADC Sensors (Thermistors)"]
        Accel["SPI Accelerometer"]
    end

    %% Connections
    Parser -->|Raw Commands| MacroEngine
    MacroEngine -->|Executed G-Code| Planner
    Planner -->|Planned Trajectories| SerDeHost
    HIL -->|Vibration Shaping| Planner
    
    SerDeHost <==>|Binary Packets| SerLink
    SerLink <==>|Frame Parsing| SerDeMCU
    
    SerDeMCU -->|Received Commands| Dispatcher
    Dispatcher -->|Enqueue Move| StepQueue
    Dispatcher -->|Set Thermal Targets| ThermalMPC
    ClockSync -->|Time Alignments| Dispatcher
    
    StepQueue -->|Drain segments| DmaStepper
    DmaStepper -->|Timer & DMA Pulses| Motors
    
    ThermalMPC -->|PWM Output| Heaters
    Thermistors -->|ADC Readings| ThermalMPC
    Thermistors -->|Thermal Limits| Safety
    
    Accel -->|SPI Samples| HIL
    Safety -->|Atomic Kill Switch| Heaters
    Safety -->|Disable Pins| Motors
```

---

## 2. Advanced Motion Control Pipeline

This diagram tracks how movements are solved using continuous blending, Jerk/Snap/Crackle-bounded kinematic profiling, and hardware-assisted stepping.

```mermaid
flowchart LR
    A["Linear Move Command"] --> B["Pythagorean-Hodograph Corner Blender"]
    B -->|Calculate PhBezier15 chord bridge| C["Newton-Raphson Parameter solver"]
    C -->|C^4 continuous geometry| D["31-Phase G4 Profile Generator"]
    D -->|Continuous Jerk, Snap & Crackle| E["SPSC Step Queue (heapless::spsc::Queue)"]
    
    subgraph MCU_IRQ ["Real-Time Hardware DMA Offloading"]
        E -->|Dequeue StepSegments| F["DmaStepEngine"]
        F -->|Active: Buffer A| G["Timer Pulse Output"]
        F -->|Inactive: Buffer B| H["Update from queue"]
        G -->|Alternate double-buffer| F
    end
```

---

## 3. Dual-Paradigm Concurrency Model

This timeline chart illustrates the scheduling priorities of async (Embassy) and hardware real-time interrupt (RTIC 2) execution.

```mermaid
gantt
    title Task & Interrupt Scheduling Priority Matrix
    dateFormat  X
    axisFormat %s
    
    section Hardware ISRs (RTIC 2)
    Stepper Timer ISR (Priority 5)     :crit, active, 0, 1
    Clock Sync Capture (Priority 4)    :active, 1, 2
    
    section Async Tasks (Embassy)
    Serial Rx/Tx Processing (Priority 3): 2, 5
    Thermal MPC updates (Priority 2)   : 5, 7
    Clock Sync Fitting (Priority 1)    : 7, 9
    Safety Supervision (Priority 1)    : 9, 10
```

---

## 4. State-Space MPC Thermal Regulation

This diagram details the Kalman Filter prediction/correction loops and feedforward mechanism inside the thermal subsystem.

```mermaid
flowchart TD
    %% Estimation Loop
    subgraph Estimator ["Kalman Filter State Estimator"]
        Predict["State Prediction: x_pred(k+1) = A*x(k) + B*u(k) + G*d(k)"]
        Update["Error Correction: x(k+1) = x_pred + K * (y_measured - y_pred)"]
    end

    %% Input Values
    T_ambient["Ambient Temp (T_ambient)"] --> Predict
    Volumetric_Flow["Volumetric Flow Rate"] --> Predict
    Prev_PWM["Previous PWM Output (u_prev)"] --> Predict
    Sensor_Read["ADC Sensor Measurement (y_measured)"] --> Update
    
    Predict -->|Predicted Sensor Temp| Update
    
    %% Controller Loop
    subgraph Controller ["MPC Power Controller"]
        Error["Calculate Internal Core Error: target_temp - T_heater_est"]
        FF["Feed-Forward: scale heater current for extrusion flow rate"]
        Clamp["Clamp PWM Output (0.0 to 1.0)"]
    end
    
    Update -->|Estimated Heater Temp (T_heater_est)| Error
    Update -->|Estimated Sensor Temp (T_sensor_est)| FF
    Volumetric_Flow --> FF
    
    Error --> Clamp
    FF --> Clamp
    Clamp -->|Output PWM Command| Heater["Physical Heater"]
    Clamp -->|Update History| Prev_PWM
```

---

## 5. Multi-MCU Clock Synchronization

This diagram details how the distributed Phase-Locked Loop (DPLL) calculates and updates master synchronization matrices in a lock-free manner.

```mermaid
sequenceDiagram
    autonumber
    participant HW as Hardware Timer Capture (Priority 4)
    participant Model as SharedClockModel (Atomic Swap)
    participant Task as Clock Model Task (Priority 1)

    HW->>Model: Query active ClockSyncModel (local_to_master)
    Model-->>HW: Returns active Model (slope & intercept)
    Note over HW: Real-time tick offset translation
    
    Task->>Task: Collect local & master tick histories
    Task->>Task: Run Recursive Least Squares regression
    Task->>Model: Update inactive model slot
    Task->>Model: Swap active index atomically
    Note over Model: Active index toggled (lock-free)
```
