use thermal::filter::{Filter, LowPassFilter, MovingAverageFilter};
use thermal::pid::Pid;
use thermal::safety::{HeaterSafety, SafetyLimits, ThermalState};
use thermal::thermistor::{SteinhartHart, Thermistor};
use float_cmp::approx_eq;

#[test]
fn test_low_pass_filter() {
    let mut filter = LowPassFilter::new(0.5, 0.0);
    filter.add_sample(10.0);
    assert!(approx_eq!(f64, filter.output(), 5.0, epsilon = 0.001));
    filter.add_sample(10.0);
    assert!(approx_eq!(f64, filter.output(), 7.5, epsilon = 0.001));
}

#[test]
fn test_moving_average_filter() {
    let mut filter: MovingAverageFilter<f32, 4> = MovingAverageFilter::new();
    filter.add_sample(10.0);
    filter.add_sample(12.0);
    filter.add_sample(11.0);
    filter.add_sample(13.0);
    assert!(approx_eq!(f32, filter.output(), 11.5, epsilon = 0.001));
    filter.add_sample(8.0); // 10.0 is dropped
    assert!(approx_eq!(f32, filter.output(), 11.0, epsilon = 0.001));
}

#[test]
fn test_thermistor_conversion_ntc_100k_b3950() {
    // Steinhart-Hart coefficients for a common NTC 100k B3950 thermistor.
    // These values are widely used in firmware like Marlin and Klipper.
    // The series resistor is set to 4.7kOhm, a common value on 3D printer boards.
    let sh = SteinhartHart {
        series_resistance: 4700.0,
        adc_max: 4095.0,
        a: 0.00078864,
        b: 0.00020845,
        c: 0.00000012506,
    };

    // Test case 1: 25°C
    // At 25°C, the thermistor's resistance is 100kOhm.
    // With a 4.7k pull-up, the expected ADC value is ~183.8
    let adc_at_25c = 183.83;
    let temp_k_25 = sh.adc_to_temperature(adc_at_25c);
    let temp_c_25 = temp_k_25 - 273.15;
    assert!(approx_eq!(f64, temp_c_25, 25.0, epsilon = 0.1), "Temp @ 25C was {:.2}", temp_c_25);

    // Test case 2: 200°C
    // At 200°C, the resistance is ~329 Ohm.
    // With a 4.7k pull-up, the expected ADC value is ~270.9
    let adc_at_200c = 270.9;
    let temp_k_200 = sh.adc_to_temperature(adc_at_200c);
    let temp_c_200 = temp_k_200 - 273.15;
    assert!(approx_eq!(f64, temp_c_200, 200.0, epsilon = 0.1), "Temp @ 200C was {:.2}", temp_c_200);
}


#[test]
fn test_pid_stability() {
    // Simple heater model for testing
    let mut temp = 25.0;
    let ambient = 25.0;
    let setpoint = 100.0;
    let mut pid = Pid::new(5.0, 0.1, 1.0, setpoint, 0.0, 1.0);
    let dt = 1.0;

    // Simulate for a while
    for _ in 0..100 {
        let output = pid.update(temp, dt);
        // Simplified model: temp change is proportional to output and heat loss
        temp += output * 2.0 - (temp - ambient) * 0.05;
    }

    // After 100s, it should be close to the setpoint
    assert!(approx_eq!(f64, temp, setpoint, epsilon = 2.0), "Final temp {} was not close to setpoint {}", temp, setpoint);
}


#[test]
fn test_safety_max_temp() {
    let limits = SafetyLimits {
        max_temp: 280.0,
        min_heat_gain_temp: 2.0,
        min_heat_gain_time_s: 10.0,
        max_deviation: 10.0,
    };
    let mut safety = HeaterSafety::new(limits);
    let state = safety.update(1.0, 285.0, 200.0, true);
    assert_eq!(state, ThermalState::Shutdown);
}


#[test]
fn test_safety_thermal_runaway() {
    let limits = SafetyLimits {
        max_temp: 280.0,
        min_heat_gain_temp: 5.0,
        min_heat_gain_time_s: 20.0,
        max_deviation: 10.0,
    };
    let mut safety = HeaterSafety::new(limits);
    // Heater on at t=0, temp=25
    safety.update(0.0, 25.0, 200.0, true);
    // After 21 seconds, temp has only risen by 2 degrees
    let state = safety.update(21.0, 27.0, 200.0, true);
    assert_eq!(state, ThermalState::Shutdown);
}

#[test]
fn test_safety_thermal_runaway_ok() {
    let limits = SafetyLimits {
        max_temp: 280.0,
        min_heat_gain_temp: 5.0,
        min_heat_gain_time_s: 20.0,
        max_deviation: 10.0,
    };
    let mut safety = HeaterSafety::new(limits);
    // Heater on at t=0, temp=25
    safety.update(0.0, 25.0, 200.0, true);
    // After 15 seconds, temp has risen by 10 degrees
    let state = safety.update(15.0, 35.0, 200.0, true);
    assert_eq!(state, ThermalState::Ok);
}

