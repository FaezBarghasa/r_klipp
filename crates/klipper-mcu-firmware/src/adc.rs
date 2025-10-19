//! # ADC Sampling
//!
//! This module is responsible for sampling the ADC (Analog-to-Digital Converter) to
//! read the values from the thermistors. The raw ADC values are then converted to
//! temperatures and made available to other tasks, such as the heater control task.
//!
//! ## Sampling and Filtering
//!
//! The ADC is sampled periodically to get the latest thermistor readings. To reduce
//! noise and improve accuracy, a filter (such as a simple moving average) is applied
//! to the raw ADC values.
//!
//! ## Temperature Conversion
//!
//! The filtered ADC values are then converted to temperatures using a lookup table or a
//! polynomial equation. The specific conversion method depends on the type of
//! thermistor being used.

use embassy_stm32::adc::Adc;
use embassy_stm32::peripherals::ADC1;
use embassy_stm32::gpio::AnyPin;


/// The ADC sampling task.
///
/// This task periodically samples the ADC to read the thermistor values.
#[embassy_executor::task]
pub async fn adc_task(_adc: ADC1, _temp_pin: AnyPin) {
    defmt::info!("ADC task started");

    // TODO: In a real implementation:
    // 1. Initialize the ADC and the thermistor pin.
    // 2. Periodically sample the ADC value.
    // 3. Apply a filter (e.g., a simple moving average).
    // 4. Convert the raw ADC value to a temperature.
    // 5. Update a shared state with the new temperature.
    loop {
        embassy_time::Timer::after(embassy_time::Duration::from_secs(1)).await;
        // let value = adc.read(&mut temp_pin);
        // defmt::info!("ADC sample: {}", value);
    }
}