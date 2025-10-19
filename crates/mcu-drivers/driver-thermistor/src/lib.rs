//! A `no_std` library for converting NTC thermistor ADC readings to temperature.
//!
//! Provides two common conversion methods:
//! 1.  **Lookup Table:** Uses linear interpolation between points in a pre-defined table.
//! 2.  **Polynomial:** Calculates temperature using polynomial coefficients (e.g., from Steinhart-Hart).
//!
//! ## Features
//! - `fixed`: Enables conversions using fixed-point numbers via the `fixed` crate.
//! - `std`: Required for running host-based tests.

#![no_std]

use num_traits::{float::Float, identities::One};

/// Defines the conversion strategy for the thermistor.
pub enum ConversionMode<'a, T> {
    /// A table of (ADC reading, Temperature) pairs. Must be sorted by ADC reading.
    Table(&'a [(u16, T)]),
    /// Polynomial coefficients in ascending order (c0, c1, c2, ...).
    Polynomial(&'a [T]),
}

/// Represents an NTC thermistor.
pub struct NtcThermistor<'a, T: Copy> {
    mode: ConversionMode<'a, T>,
}

impl<'a, T: Float + One + Copy> NtcThermistor<'a, T> {
    /// Creates a new thermistor instance with the given conversion mode.
    pub fn new(mode: ConversionMode<'a, T>) -> Self {
        Self { mode }
    }

    /// Converts an ADC reading to a temperature value.
    ///
    /// Returns `None` if the ADC reading is out of the bounds of a lookup table.
    pub fn temperature(&self, adc_reading: u16) -> Option<T> {
        match self.mode {
            ConversionMode::Table(table) => self.temp_from_table(adc_reading, table),
            ConversionMode::Polynomial(coeffs) => self.temp_from_poly(adc_reading, coeffs),
        }
    }

    /// Calculates temperature from a lookup table using linear interpolation.
    fn temp_from_table(&self, adc_reading: u16, table: &[(u16, T)]) -> Option<T> {
        if table.is_empty() {
            return None;
        }

        // Find the position in the table.
        let pos = table.binary_search_by_key(&adc_reading, |(adc, _)| *adc);

        match pos {
            // Exact match
            Ok(index) => Some(table[index].1),
            // No exact match, need to interpolate
            Err(index) => {
                if index == 0 || index >= table.len() {
                    // Out of bounds
                    None
                } else {
                    // Interpolate between (index-1) and (index)
                    let (adc1, temp1) = table[index - 1];
                    let (adc2, temp2) = table[index];

                    let adc1_t = T::from(adc1).unwrap();
                    let adc2_t = T::from(adc2).unwrap();
                    let reading_t = T::from(adc_reading).unwrap();

                    let t = (reading_t - adc1_t) / (adc2_t - adc1_t);
                    let temp = temp1 + t * (temp2 - temp1);
                    Some(temp)
                }
            }
        }
    }

    /// Calculates temperature from polynomial coefficients.
    fn temp_from_poly(&self, adc_reading: u16, coeffs: &[T]) -> Option<T> {
        let x = T::from(adc_reading).unwrap();
        let mut temp = T::zero();
        let mut x_pow = T::one();

        for &coeff in coeffs {
            temp = temp + coeff * x_pow;
            x_pow = x_pow * x;
        }
        Some(temp)
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    // A sample lookup table for a common NTC thermistor.
    // (ADC Reading on 12-bit ADC, Temperature in °C)
    const TEMP_TABLE: [(u16, f32); 5] = [
        (356, 100.0),
        (852, 50.0),
        (1825, 25.0),
        (3084, 0.0),
        (3828, -20.0),
    ];

    // Sample polynomial coefficients (dummy values for testing).
    const POLY_COEFFS: [f64; 3] = [123.45, -0.0678, 0.0000123];

    #[test]
    fn test_table_exact_match() {
        let thermistor = NtcThermistor::new(ConversionMode::Table(&TEMP_TABLE));
        assert_eq!(thermistor.temperature(1825), Some(25.0));
        assert_eq!(thermistor.temperature(356), Some(100.0));
    }

    #[test]
    fn test_table_interpolation() {
        let thermistor = NtcThermistor::new(ConversionMode::Table(&TEMP_TABLE));
        // Exactly halfway between 50°C and 25°C
        let adc_mid = 852 + (1825 - 852) / 2;
        let temp = thermistor.temperature(adc_mid).unwrap();
        assert!((temp - 37.5).abs() < 1e-6);
    }

    #[test]
    fn test_table_out_of_bounds() {
        let thermistor = NtcThermistor::new(ConversionMode::Table(&TEMP_TABLE));
        assert_eq!(thermistor.temperature(100), None); // Below table
        assert_eq!(thermistor.temperature(4000), None); // Above table
    }

    #[test]
    fn test_polynomial_conversion() {
        let thermistor = NtcThermistor::new(ConversionMode::Polynomial(&POLY_COEFFS));
        let adc = 2000.0;
        let expected_temp = POLY_COEFFS[0] + POLY_COEFFS[1] * adc + POLY_COEFFS[2] * adc * adc;
        let temp = thermistor.temperature(2000).unwrap();
        assert!((temp - expected_temp).abs() < 1e-9);
    }
}
