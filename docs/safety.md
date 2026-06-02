# Safety Subsystem

The safety of the user and the machine is the highest priority in the Klipper in Rust firmware. This document details the multi-layered safety subsystem designed to prevent accidents, detect faults, and ensure the printer always operates in a known, safe state.

---

## 1. Guiding Principles

*   **Fail-Safe Design**: In the event of a fault, the firmware will always transition the hardware to a safe state (e.g., heaters off, motors disabled).
*   **Redundancy**: Critical safety checks are performed by multiple components to provide redundancy.
*   **Hardware and Software Interlocks**: Safety is enforced by a combination of hardware features (e.g., watchdog timers) and software logic.
*   **User Notification**: When a safety event occurs, the firmware will make a best effort to inform the user via the Klipper host.

---

## 2. Key Safety Features

### 2.1. Watchdog Timers

*   **Hardware Watchdog (IWDG)**: The MCU's Independent Watchdog is used to guard against firmware lockups. The main loop must "pet" the watchdog at regular intervals. If it fails to do so (e.g., due to an infinite loop), the watchdog will reset the MCU, bringing it back to a known state.
*   **Software Task Watchdogs**: Individual high-priority tasks (e.g., the thermal control loop) have their own software watchdog check-ins. If a task fails to check in before its configured deadline (checked via `check_task_stalls`), the safety monitor triggers an emergency stop.

### 2.2. Thermal Safety

Overheating is a significant fire hazard. The `ThermalMonitor` in [safety.rs](file:///home/jrad/RustroverProjects/r_klipp-workspace/r_klipp/crates/klipper-mcu-firmware/src/safety.rs) implements strict verification loops:

*   **Plausible Limits**: Ensures thermistor measurements do not cross absolute minimum and maximum temperature bounds (`min_temp_limit` and `max_temp_limit`). Values outside this range (e.g. open/shorted sensors) trigger faults.
*   **Thermal Runaway Protection**: Monitors the rate of temperature change over time. If a heater's temperature increases faster than the maximum allowable rate (`max_rate_celsius_per_sec`), it indicates a dangerous thermal runaway condition (e.g., a detached heater cartridge) and halts operation.

### 2.3. Motion and Driver Safety

*   **Endstop Monitoring**: The firmware continuously monitors the state of the endstops. If an endstop is triggered unexpectedly, it can halt motion.
*   **Stepper Driver Fault Detection**: For smart stepper drivers (e.g. TMC2209), the firmware monitors fault status pins. If a driver reports a fault, its corresponding bits are flagged in a bitmask. Any non-zero mask triggers a shutdown.
*   **Maximum Velocity limits**: Guard boundaries restrict moves that exceed physical kinematics limits.

---

## 3. Emergency Stop (E-Stop)

An emergency stop is the ultimate safety mechanism. It is coordinated by the `SafetyMonitor` structure.

### 3.1. Triggers
An E-stop is triggered by:
- A host command (`M112` or `Emergency_stop` packet).
- The `SafetyMonitor` detecting a critical fault (`SafetyError`).
- A physical E-stop button.

The concrete types of faults identified by the monitor are defined in the `SafetyError` enum:

```rust
pub enum SafetyError {
    ThermalRunaway { heater_id: usize, rate_of_change: f32 },
    TempTooLow { heater_id: usize, temp: f32 },
    TempTooHigh { heater_id: usize, temp: f32 },
    StepperDriverFault { driver_mask: u8 },
    TaskStalled { task_id: usize },
}
```

### 3.2. Actions
When an E-stop is triggered, the firmware performs the following actions in a hard real-time context:

1.  **Transition Global Atomic Flag**: The global atomic boolean state `emergency_stop_active` is swapped to `true` in a thread-safe, lock-free manner.
2.  **Immediately disable all heaters** by turning off their PWM outputs and disabling their GPIO pins.
3.  **Immediately disable all stepper motors** by deactivating their `enable` pins.
4.  **Halt all motion planning** and clear the motion queue.
5.  Send a `Shutdown` message to the host.
6.  Enter an infinite loop, requiring a hardware reset to recover.

---

## 4. Implementation Details

- **SafetyMonitor Task**: The main safety supervisor is defined in the `SafetyMonitor` struct:
  ```rust
  pub struct SafetyMonitor<'a, T, const NUM_HEATERS: usize, const NUM_TASKS: usize> {
      thermal_monitors: [ThermalMonitor; NUM_HEATERS],
      watchdog: IndependentWatchdog<'a, T>,
      emergency_stop_active: AtomicBool,
      last_check_in: [Instant; NUM_TASKS],
      task_deadlines: [Duration; NUM_TASKS],
  }
  ```
- **Failsafe Executions**: Task loops check `is_emergency_stop_active` at the start of each execution step. If active, they immediately exit or drop into safe standby modes.
- **Portability**: Leverages traits provided by `embedded-hal` to ensure safety supervisor routines can run across different target boards.
