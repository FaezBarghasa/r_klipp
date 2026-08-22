# `r_klipp_safety`: Hardware Supervisor & Real-Time Safety Interlocks

`r_klipp_safety` provides hardware watchdog management, emergency stop coordination, touch probe latch supervisors, and enclosure door safety interlocks.

---

## 🛡️ Key Components

### 1. `IndependentWatchdog` (IWDG)
- Enforces heartbeat verification with configurable timeout thresholds ($<500\text{ ms}$).
- Triggers hardware reset and de-energizes all motor and heater outputs upon communication stalls.

### 2. `ProbeSupervisor` (`G38.2`)
- Monitors high-speed touch probes and tool setters.
- Latches electrical contact within **one timer tick**, commanding instant trajectory deceleration to preserve probe indicators.

### 3. `EnclosureInterlock` (CNC Safety)
- Monitors physical machine enclosure doors.
- Disables spindle power and drops axis enable pins if the enclosure is opened while spindle RPM is non-zero.

---

## 🧪 Testing
```bash
cargo test -p safety
```
