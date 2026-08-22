# `r_klipp_thermal`: State-Space MPC & PID Thermal Management

`r_klipp_thermal` is a `no_std`, high-precision thermal control crate implementing state-space Model Predictive Control (MPC), Kalman-filtered state estimation, anti-windup PID control, and multi-tier thermal runaway safety guards.

---

## 🌡️ Features & Capabilities

### 1. State-Space Model Predictive Control (MPC)
- Estimates internal heater core temperature and sensor delays via state-space matrices:
  $$\mathbf{x}_{k+1} = \mathbf{A} \mathbf{x}_k + \mathbf{B} u_k + \mathbf{K} (y_{\text{sensor}} - \mathbf{C} \mathbf{x}_k)$$
- Feedforward extrusion flow rate compensation: pre-emptively increases heater duty cycle during high-volumetric extrusion moves to eliminate temperature dips.

### 2. High-Performance PID with Anti-Windup
- Standard Proportional-Integral-Derivative loop with derivative low-pass filtering and integral clamping to prevent overshoot.

### 3. Sensor Conversion Models
- **NTC Thermistor**: Steinhart-Hart and $\beta$-parameter ($B=3950$, $100\text{k}\Omega$) lookup and analytical conversion.
- **Thermocouple / PT100 / PT1000**: Linearized resistance and voltage conversion.
- **Moving Average & Low-Pass Filtering**: Suppresses ADC noise and switching transients.

### 4. Zero-Compromise Safety Guards
- **Thermal Runaway Detection**: Automatically trips if heater fails to climb during heating or deviates by $> \pm 5^\circ\text{C}$ in steady state.
- **Disconnected / Short-Circuit Sensor Detection**: Shuts down heaters if $T < 0^\circ\text{C}$ or $T > T_{\text{max}}$.

---

## 🧪 Testing
```bash
cargo test -p thermal
```
