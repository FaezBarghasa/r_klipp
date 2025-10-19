//! Thermistor models for converting ADC values to temperature.

use num_traits::Float;

/// A trait for thermistor models.
pub trait Thermistor<Adc, Temp> {
    /// Converts an ADC reading to a temperature value.
    fn adc_to_temperature(&self, adc_value: Adc) -> Temp;
}

/// A thermistor model using the Steinhart-Hart equation.
///
/// This is a common model for NTC thermistors. The coefficients (a, b, c) are
/// typically provided by the thermistor manufacturer or can be calculated from

/// resistance-temperature data points.
/// T = 1 / (a + b * ln(R) + c * (ln(R))^3)
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SteinhartHart<F: Float> {
    /// The resistance of the pull-up or pull-down resistor in the voltage divider.
    pub series_resistance: F,
    /// The maximum ADC value (e.g., 4095 for a 12-bit ADC).
    pub adc_max: F,
    /// Steinhart-Hart coefficient A.
    pub a: F,
    /// Steinhart-Hart coefficient B.
    pub b: F,
    /// Steinhart-Hart coefficient C.
    pub c: F,
}

impl<F: Float> SteinhartHart<F> {
    /// Creates a new `SteinhartHart` model.
    pub fn new(series_resistance: F, adc_max: F, a: F, b: F, c: F) -> Self {
        Self {
            series_resistance,
            adc_max,
            a,
            b,
            c,
        }
    }

    /// Calculates the thermistor's resistance from an ADC reading.
    pub fn adc_to_resistance(&self, adc_value: F) -> F {
        // Ensure adc_value is not equal to adc_max to avoid division by zero.
        if adc_value >= self.adc_max {
            return F::zero(); // Effectively infinite temperature, resistance is zero
        }
        // R = R_series * (ADC_max / ADC - 1)
        self.series_resistance * (self.adc_max / adc_value - F::one())
    }
}

impl<F: Float> Thermistor<F, F> for SteinhartHart<F> {
    /// Converts an ADC reading to temperature in Kelvin.
    fn adc_to_temperature(&self, adc_value: F) -> F {
        if adc_value <= F::zero() {
            return F::zero(); // Effectively infinite resistance, zero Kelvin
        }
        let resistance = self.adc_to_resistance(adc_value);
        if resistance <= F::zero() {
            // Very high temperature
            return F::infinity();
        }

        let ln_r = resistance.ln();
        let temp_inv = self.a + self.b * ln_r + self.c * ln_r.powi(3);

        if temp_inv <= F::zero() {
            F::infinity()
        } else {
            F::one() / temp_inv
        }
    }
}
