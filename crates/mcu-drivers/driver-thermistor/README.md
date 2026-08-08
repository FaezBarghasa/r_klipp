# Klipper in Rust: Thermistor Driver

## Overview

This crate provides a driver for reading thermistors and converting their ADC readings into temperature values. It is a critical component of the thermal management subsystem.

## Features

*   **Steinhart-Hart Model**: Uses the Steinhart-Hart equation for accurate temperature conversion.
*   **Thermistor Tables**: Can be configured with pre-computed lookup tables for faster temperature calculations.
*   **Sensor Type Support**: Supports different thermistor types by allowing the configuration of their beta values and nominal resistances.
*   **ADC Abstraction**: The driver is generic over the ADC peripheral, making it compatible with any ADC that implements the `embedded-hal` ADC traits.

## Usage

```rust
use driver_thermistor::{Thermistor, ThermistorConfig};
use embedded_hal::adc::OneShot;

fn setup_thermistor<ADC, PIN>(adc: &mut ADC, pin: PIN) -> Thermistor<ADC, PIN>
where
    ADC: OneShot<ADC, u16, PIN>,
    PIN: embedded_hal::adc::Channel<ADC>,
{
    let config = ThermistorConfig {
        beta: 3950.0,
        nominal_resistance: 100_000.0,
        nominal_temperature: 25.0,
        shunt_resistance: 4700.0,
    };
    Thermistor::new(adc, pin, config)
}
```
