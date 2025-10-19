#![cfg(test)]

use crate::safety::{SafetyError, ThermalMonitor};
use embassy_time::{Duration, Instant};

// Helper to create a default monitor for tests
fn create_test_monitor() -> ThermalMonitor {
    ThermalMonitor::new(
        5.0,    // 5 °C/sec runaway limit
        -50.0,  // Min temp limit
        300.0,  // Max temp limit
        25.0,   // Initial temp
    )
}

#[test]
fn test_thermal_runaway_detection() {
    let mut monitor = create_test_monitor();
    let mut now = Instant::now();

    // First check establishes baseline
    monitor.check(0, 25.0).unwrap();
    now += Duration::from_secs(1);

    // Simulate a rapid 6°C rise in 1 second
    let temp = 25.0 + 6.0;
    let result = monitor.check(0, temp);

    match result {
        Err(SafetyError::ThermalRunaway { heater_id, rate_of_change }) => {
            assert_eq!(heater_id, 0);
            assert!((rate_of_change - 6.0).abs() < 0.01, "Rate was {}", rate_of_change);
        }
        _ => panic!("Expected ThermalRunaway error, got {:?}", result),
    }
}

#[test]
fn test_thermal_runaway_is_not_triggered_on_normal_heating() {
    let mut monitor = create_test_monitor();
    let mut now = Instant::now();

    // First check
    monitor.check(0, 25.0).unwrap();
    now += Duration::from_secs(1);

    // Simulate a normal 2°C rise
    let temp = 25.0 + 2.0;
    assert!(monitor.check(0, temp).is_ok());

    // Another normal 2°C rise
    now += Duration::from_secs(1);
    let temp2 = temp + 2.0;
    assert!(monitor.check(0, temp2).is_ok());
}

#[test]
fn test_sensor_failure_too_high() {
    let mut monitor = create_test_monitor();
    let result = monitor.check(1, 301.0);

    match result {
        Err(SafetyError::TempTooHigh { heater_id, temp }) => {
            assert_eq!(heater_id, 1);
            assert_eq!(temp, 301.0);
        }
        _ => panic!("Expected TempTooHigh error, got {:?}", result),
    }
}

#[test]
fn test_sensor_failure_too_low() {
    let mut monitor = create_test_monitor();
    let result = monitor.check(0, -51.0);

    match result {
        Err(SafetyError::TempTooLow { heater_id, temp }) => {
            assert_eq!(heater_id, 0);
            assert_eq!(temp, -51.0);
        }
        _ => panic!("Expected TempTooLow error, got {:?}", result),
    }
}

#[test]
fn test_short_interval_does_not_trigger_runaway() {
    let mut monitor = create_test_monitor();
    let mut now = Instant::now();

    monitor.check(0, 25.0).unwrap();

    // Simulate a very short interval (50ms) with a large temp jump
    now += Duration::from_millis(50);
    let temp = 25.0 + 3.0;

    // The check should pass because the interval is too short for a reliable reading
    assert!(monitor.check(0, temp).is_ok());
}
