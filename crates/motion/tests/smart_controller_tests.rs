#![cfg(test)]

use motion::smart_controller::{SmartController, ControlMode};
use motion::tuner_trait::TuningDomain;

#[test]
fn test_thermal_tuning() {
    let mut smart_controller = SmartController::new(TuningDomain::Temperature, 0.0, 0.0, 0.0);
    smart_controller.start_autotune(100.0);

    let mut temp = 25.0;
    let mut time = 0.0;

    for _ in 0..10000 {
        let output = smart_controller.compute(temp, time);
        temp += (output - (temp - 25.0) * 0.1) * 0.1;
        time += 0.1;
    }

    assert_eq!(smart_controller.get_mode(), ControlMode::Run);
    let (kp, ki, kd) = smart_controller.get_gains();
    assert!(kp > 0.0);
    assert!(ki > 0.0);
    assert!(kd > 0.0);
}

#[test]
fn test_motion_tuning() {
    let mut smart_controller = SmartController::new(TuningDomain::Motion, 0.0, 0.0, 0.0);
    smart_controller.start_autotune(50.0);

    let mut pos = 0.0;
    let mut vel = 0.0;
    let mass = 1.0;
    let mut time = 0.0;

    for _ in 0..500 {
        let force = smart_controller.compute(pos, time);
        let accel = force / mass;
        vel += accel * 0.01;
        pos += vel * 0.01;
        time += 0.01;
    }

    assert_eq!(smart_controller.get_mode(), ControlMode::Run);
    let (kp, ki, kd) = smart_controller.get_gains();
    assert!(kp > 0.0);
    assert!(ki > 0.0);
    assert!(kd >= 0.0);
}

#[test]
fn test_inner_loop_transition() {
    let mut smart_controller = SmartController::new(TuningDomain::Temperature, 0.0, 0.0, 0.0);
    smart_controller.start_autotune(100.0);

    let mut temp = 25.0;
    let mut time = 0.0;

    while smart_controller.get_mode() == ControlMode::AutoTune {
        let output = smart_controller.compute(temp, time);
        temp += (output - (temp - 25.0) * 0.1) * 0.1;
        time += 0.1;
    }

    assert_eq!(smart_controller.get_mode(), ControlMode::Run);
    let (kp, ki, kd) = smart_controller.get_gains();
    assert!(kp > 0.0);
    assert!(ki > 0.0);
    assert!(kd > 0.0);
}