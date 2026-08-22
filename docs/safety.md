# Safety Engine Architecture & Safety Constraints

> [!CAUTION]
> CNC milling spindles, high-temperature 3D printer hotends, and high-speed robotic pick-and-place gantries carry severe physical safety hazards. The `r_klipp` safety engine operates as a zero-compromise, fail-safe layer enforced at compile time and verified in real time.

---

## 🛡️ 1. Multi-Layer Safety Architecture

```mermaid
graph TD
    subgraph Hardware Layer
        Watchdog[Hardware IWDG Watchdog]
        EstopPin[Physical E-Stop Circuit]
        DoorSensor[Enclosure Interlock Switches]
        ProbePin[Touch Probing Latches]
    end

    subgraph Real-Time Supervisor (no_std)
        WatchdogFeeder[Watchdog Supervisor 100Hz]
        ThermalGuard[Thermal Runaway & Max-Temp Guard]
        ProbeSupervisor[Single-Tick Probe Latch Supervisor]
        InterlockGuard[Spindle & Enclosure Interlock Guard]
    end

    subgraph Motion Planner
        KinematicsSafety[Axis Limit & Head Clearance Validation]
        StallGuard[TMC2240 Motor Torque Surveillance]
        PlannerQueue[Trajectory Ramp-Down on Abort]
    end

    DoorSensor --> InterlockGuard
    ProbePin --> ProbeSupervisor
    ThermalGuard --> EstopPin
    InterlockGuard --> EstopPin
    WatchdogFeeder --> Watchdog
    StallGuard --> PlannerQueue
```

---

## ⚡ 2. Safety Invariants & Enforcement

### 2.1 Hardware Independent Watchdog (IWDG)
- **Timeout Period**: $500\text{ ms}$ max allowed heartbeat latency.
- **Enforcement**: If the real-time planner or communication thread fails to feed the watchdog within the configured window, the MCU hardware automatically resets all GPIO outputs to high-impedance safe state, de-energizing stepper drivers, heater MOSFETs, and spindle relays.

### 2.2 Thermal Runaway Protection
- **Max Temperature Limit**: Immediate emergency shutdown if measured temperature exceeds $T_{\text{max}}$.
- **Minimum Temperature Sensor Check**: Detects disconnected or short-circuited thermistors ($T < 0^\circ\text{C}$ or $R \to \infty$).
- **Thermal Runaway Detection**:
  - Heating phase: If target is not reached within $\Delta t_{\text{heat\_timeout}}$ under full duty cycle.
  - Steady-state phase: If current temperature deviates from target by $> \pm 5^\circ\text{C}$ for longer than $15\text{ seconds}$.

### 2.3 Single-Tick Probing Contact Supervisor (`G38.2`)
- Probing moves decelerate until electrical contact is detected.
- Contact signal is latched within **one timer tick** ($< 10\,\mu\text{s}$), freezing coordinate positions immediately to prevent dial indicator or probe tip destruction.

### 2.4 High-Speed Spindle & Enclosure Interlocks
- **CNC Safety Invariant**: The VFD spindle cannot be energized if the enclosure door is open.
- If the enclosure door switch opens while the spindle is running at $>0\text{ RPM}$, the safety supervisor immediately triggers an E-Stop, commands VFD dynamic braking, and drops all axis enable lines.

### 2.5 PnP Toolhead Collision Avoidance
- Dual-head pick-and-place toolheads enforce a dynamic physical clearance zone $\Delta X_{\text{min}}$ between Head 1 and Head 2.
- The kinematics engine rejects moves that violate clearance before trajectory calculation begins.

---

## 🚨 3. Emergency Stop (E-Stop) State Machine

| Current State | Trigger Event | Action Taken | Next State |
| :--- | :--- | :--- | :--- |
| **Operational** | E-Stop Button Pressed | Cut PWM, disable steppers, flush queues | **E-Stop Latched** |
| **Operational** | Thermal Runaway | Cut all heater MOSFETs, broadcast error | **E-Stop Latched** |
| **Operational** | Door Interlock Opened (CNC) | Brake spindle, kill motion | **E-Stop Latched** |
| **Operational** | Watchdog Expiration | Hardware MCU Reset | **Hardware Reset** |
| **E-Stop Latched** | `M112` / `M999` Reset | System audit, confirm safety cleared | **Operational** |