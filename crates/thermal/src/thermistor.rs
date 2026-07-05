pub struct SteinhartHartConfig {
    pub reference_resistance_ohms: f32,
    pub beta: Option<f32>, // Beta parameter for simpler calibration
    pub c_coeffs: Option<[f32; 3]>, // A, B, C coefficients for Steinhart-Hart
}

pub struct Thermistor {
    config: SteinhartHartConfig,
    pull_up_resistance_ohms: f32, // Resistance of the pull-up resistor in the voltage divider
    adc_max_value: f32,           // Maximum value of the ADC (e.g., 4095 for 12-bit ADC)
    ema_alpha: f32,               // Alpha for Exponential Moving Average filter
    current_temperature_ema: f32, // Current filtered temperature
}

impl Thermistor {
    pub fn new(
        config: SteinhartHartConfig,
        pull_up_resistance_ohms: f32,
        adc_max_value: f32,
        ema_alpha: f32,
        initial_temperature: f32,
    ) -> Self {
        Self {
            config,
            pull_up_resistance_ohms,
            adc_max_value,
            ema_alpha,
            current_temperature_ema: initial_temperature,
        }
    }

    /// Converts raw ADC reading to thermistor resistance.
    /// Assumes a voltage divider with thermistor to ground and pull-up to VCC.
    pub fn adc_to_resistance(&self, adc_reading: u16) -> f32 {
        let voltage_at_junction = adc_reading as f32; // Assuming ADC reading is proportional to voltage
        if voltage_at_junction == 0.0 {
            // Avoid division by zero if ADC reads 0
            return f32::MAX;
        }
        self.pull_up_resistance_ohms * (self.adc_max_value / voltage_at_junction - 1.0)
    }

    /// Converts thermistor resistance to temperature in Celsius using Steinhart-Hart or Beta parameter.
    pub fn resistance_to_temperature(&self, resistance_ohms: f32) -> f32 {
        let r_t = resistance_ohms;
        let r_0 = self.config.reference_resistance_ohms;

        if let Some(beta) = self.config.beta {
            // Beta parameter equation
            let inv_t_0 = 1.0 / (25.0 + 273.15); // Reference temperature (T0) in Kelvin (25 C)
            let inv_t = inv_t_0 + (1.0 / beta) * (r_t / r_0).ln();
            (1.0 / inv_t) - 273.15 // Convert back to Celsius
        } else if let Some(c_coeffs) = self.config.c_coeffs {
            // Steinhart-Hart equation: 1/T = A + B*ln(R) + C*(ln(R))^3
            let ln_r = r_t.ln();
            let inv_t = c_coeffs[0] + c_coeffs[1] * ln_r + c_coeffs[2] * ln_r.powi(3);
            (1.0 / inv_t) - 273.15 // Convert back to Celsius
        } else {
            // Fallback or error if no calibration method is provided
            0.0 // Or handle error appropriately
        }
    }

    /// Applies an Exponential Moving Average filter to the temperature.
    pub fn update_temperature_ema(&mut self, new_temperature: f32) -> f32 {
        self.current_temperature_ema =
            self.ema_alpha * new_temperature + (1.0 - self.ema_alpha) * self.current_temperature_ema;
        self.current_temperature_ema
    }

    /// Reads ADC, converts to resistance, then to temperature, and applies EMA filter.
    pub fn read_and_filter_temperature(&mut self, adc_reading: u16) -> f32 {
        let resistance = self.adc_to_resistance(adc_reading);
        let temperature = self.resistance_to_temperature(resistance);
        self.update_temperature_ema(temperature)
    }
}