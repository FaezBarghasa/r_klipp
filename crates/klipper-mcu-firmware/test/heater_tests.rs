#![cfg(test)]

use crate::fixed_point::Fixed16_16;
use crate::heater::PidController;

// --- Fixed-Point Math Tests ---

#[test]
fn fixed_from_to_float() {
    let f1 = 123.456;
    let fix1 = Fixed16_16::from_float(f1);
    // Check that the conversion is within a reasonable tolerance
    assert!((fix1.to_float() - f1).abs() < 0.0001);

    let f2 = -50.75;
    let fix2 = Fixed16_16::from_float(f2);
    assert!((fix2.to_float() - f2).abs() < 0.0001);
}

#[test]
fn fixed_add_sub() {
    let a = Fixed16_16::from_float(10.5);
    let b = Fixed16_16::from_float(5.25);
    assert_eq!((a + b).to_float(), 15.75);
    assert_eq!((a - b).to_float(), 5.25);
}

#[test]
fn fixed_mul_div() {
    let a = Fixed16_16::from_float(10.5);
    let b = Fixed16_16::from_float(2.0);
    assert!(((a * b).to_float() - 21.0).abs() < 0.0001);
    assert!(((a / b).to_float() - 5.25).abs() < 0.0001);
}

#[test]
fn fixed_mul_fractional() {
    let a = Fixed16_16::from_float(20.0);
    let b = Fixed16_16::from_float(0.25);
    assert!(((a * b).to_float() - 5.0).abs() < 0.0001);
}


// --- PID Controller Tests ---

#[test]
fn test_pid_controller_proportional_term() {
    let max_duty = 1000;
    let mut pid = PidController::new(10.0, 0.0, 0.0, max_duty);
    let dt = Fixed16_16::from_float(0.1);

    // Large error, should clamp to max_duty
    let error1 = Fixed16_16::from_float(50.0); // 50°C error
    let output1 = pid.update(error1, dt);
    // Expected: Kp * error = 10.0 * 50.0 = 500.0. Clamped to max_duty
    assert_eq!(output1, max_duty);

    // Small error, should be proportional
    let error2 = Fixed16_16::from_float(5.0); // 5.0°C error
    let output2 = pid.update(error2, dt);
    // Expected: 10.0 * 5.0 = 50.0
    assert!((output2 as f32 - 50.0).abs() < 1.0); // Allow for fixed-point rounding
}

#[test]
fn test_pid_controller_integral_term() {
    let max_duty = 1000;
    let mut pid = PidController::new(0.0, 2.0, 0.0, max_duty);
    let error = Fixed16_16::from_float(5.0); // 5.0°C error
    let dt = Fixed16_16::from_float(1.0);

    // First update
    let output1 = pid.update(error, dt);
    // Integral = 5.0 * 1.0 = 5.0. Output = Ki * integral = 2.0 * 5.0 = 10.0
    assert!((output1 as f32 - 10.0).abs() < 1.0);

    // Second update with same error
    let output2 = pid.update(error, dt);
    // Integral = 5.0 + 5.0 * 1.0 = 10.0. Output = Ki * integral = 2.0 * 10.0 = 20.0
    assert!((output2 as f32 - 20.0).abs() < 1.0);
}

#[test]
fn test_pid_anti_windup() {
    let max_duty = 1000;
    // Ki is high, which would normally cause massive integral buildup
    let mut pid = PidController::new(0.1, 50.0, 0.0, max_duty);
    let error = Fixed16_16::from_float(10.0); // Large error
    let dt = Fixed16_16::from_float(1.0);

    // Run for 10 iterations. Without anti-windup, integral would be huge.
    for _ in 0..10 {
        pid.update(error, dt);
    }

    // Check internal integral state. Should be clamped.
    // integral_max = output_max / ki = 1000.0 / 50.0 = 20.0
    let internal_integral = pid.integral;
    assert!((internal_integral.to_float() - 20.0).abs() < 0.1);
}

#[test]
fn test_pid_derivative_term() {
    let max_duty = 1000;
    let mut pid = PidController::new(0.0, 0.0, 3.0, max_duty);
    let dt = Fixed16_16::from_float(0.5);

    // First error is 2.0
    pid.update(Fixed16_16::from_float(2.0), dt);
    // Second error is 1.0 (temp is rising towards setpoint)
    let output = pid.update(Fixed16_16::from_float(1.0), dt);

    // Derivative = (current_error - prev_error) / dt = (1.0 - 2.0) / 0.5 = -2.0
    // D term = Kd * derivative = 3.0 * -2.0 = -6.0. Output is clamped to 0.0.
    assert_eq!(output, 0);
}
