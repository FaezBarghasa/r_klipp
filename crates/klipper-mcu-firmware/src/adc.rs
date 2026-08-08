//! # ADC Sampling
//!
//! This module is responsible for sampling the ADC (Analog-to-Digital Converter) to
//! read the values from the thermistors. The raw ADC values are then converted to
//! temperatures and made available to other tasks, such as the heater control task.

use embassy_stm32::peripherals::ADC1;
use embassy_stm32::adc::Adc;
use crate::heater::HeaterSharedState;
use crate::fixed_point::Fixed16_16;

/// Steinhart-Hart/Beta parameter temperature conversion.
/// Converts raw 12-bit ADC value to temperature in Celsius.
pub fn adc_to_celsius(raw_adc: u16) -> f32 {
    if raw_adc == 0 {
        return -273.15; // Absolute zero / error state
    }
    if raw_adc >= 4095 {
        return 999.0; // Short circuit / error state
    }
    
    let r_pullup = 4700.0; // 4.7k Ohm pullup resistor
    let adc_max = 4095.0;
    
    // Calculate thermistor resistance: R = R_pu * (raw_adc / (adc_max - raw_adc))
    let r_therm = r_pullup * (raw_adc as f32 / (adc_max - raw_adc as f32));
    
    // Beta parameters for standard 100k NTC 3950:
    let beta = 3950.0;
    let r0 = 100000.0;
    let t0 = 298.15; // 25 C in Kelvin
    
    // 1/T = 1/T0 + 1/Beta * ln(R/R0)
    let ln_r_r0 = libm::logf(r_therm / r0);
    let inv_t = (1.0 / t0) + (1.0 / beta) * ln_r_r0;
    let t_kelvin = 1.0 / inv_t;
    
    t_kelvin - 273.15 // Celsius
}

/// The ADC sampling task.
///
/// This task periodically samples the ADC to read the thermistor values.
#[embassy_executor::task]
pub async fn adc_task(
    adc_periph: ADC1,
    mut temp_pin: embassy_stm32::peripherals::PA0,
    state: &'static HeaterSharedState,
) {
    defmt::info!("ADC task started");

    let mut adc = Adc::new(adc_periph, &mut embassy_time::Delay);
    
    // 12-bit resolution is standard on STM32F407, but set explicitly if needed
    adc.set_sample_time(embassy_stm32::adc::SampleTime::Cycles480);

    // Read initial sample to initialize filter history
    let initial_val = adc.read(&mut temp_pin);
    const FILTER_SAMPLES: usize = 8;
    let mut history = [initial_val; FILTER_SAMPLES];
    let mut index = 0;

    loop {
        // Sample every 250ms (4Hz sampling rate)
        embassy_time::Timer::after(embassy_time::Duration::from_millis(250)).await;

        let raw_val = adc.read(&mut temp_pin);
        history[index] = raw_val;
        index = (index + 1) % FILTER_SAMPLES;

        // Apply moving average filter
        let mut sum = 0u32;
        for &val in history.iter() {
            sum += val as u32;
        }
        let filtered_val = (sum / FILTER_SAMPLES as u32) as u16;

        // Convert raw ADC value to Celsius
        let temp_celsius = adc_to_celsius(filtered_val);

        // Update shared state with fixed-point temperature
        let mut current_temp_lock = state.current_temp.lock().await;
        *current_temp_lock = Fixed16_16::from_float(temp_celsius);

        defmt::trace!("ADC Raw: {}, Filtered: {}, Temp: {} C", raw_val, filtered_val, temp_celsius);
    }
}