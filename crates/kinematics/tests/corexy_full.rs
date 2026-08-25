use kinematics::{CoreXYKinematics, Kinematics};
use nalgebra::Vector3;

#[test]
fn test_corexy_pure_x_motion() {
    let mut corexy = CoreXYKinematics;

    // In CoreXY: Move purely along +X (say X=10, Y=0, Z=0)
    // Both A and B motors must move equally in positive direction
    let target = Vector3::new(10.0, 0.0, 5.0);
    let motors = corexy.inverse_kinematics(target).expect("Valid IK target");

    // A = X + Y = 10 + 0 = 10
    // B = X - Y = 10 - 0 = 10
    // Z = 5
    assert_eq!(motors.x, 10.0, "Motor A should be 10.0 for pure X move");
    assert_eq!(motors.y, 10.0, "Motor B should be 10.0 for pure X move");
    assert_eq!(motors.z, 5.0, "Motor Z should be 5.0");

    // Test FK roundtrip
    let reconstructed = corexy.forward_kinematics(motors).expect("Valid FK");
    assert!((reconstructed.x - target.x).abs() < 1e-6);
    assert!((reconstructed.y - target.y).abs() < 1e-6);
    assert!((reconstructed.z - target.z).abs() < 1e-6);
}

#[test]
fn test_corexy_pure_y_motion() {
    let mut corexy = CoreXYKinematics;

    // Move purely along +Y (X=0, Y=10, Z=0)
    // Motor A = X + Y = 10, Motor B = X - Y = -10
    let target = Vector3::new(0.0, 10.0, 0.0);
    let motors = corexy.inverse_kinematics(target).expect("Valid IK target");

    assert_eq!(motors.x, 10.0, "Motor A should be +10.0 for pure +Y move");
    assert_eq!(motors.y, -10.0, "Motor B should be -10.0 for pure +Y move");

    let reconstructed = corexy.forward_kinematics(motors).expect("Valid FK");
    assert!((reconstructed.x - target.x).abs() < 1e-6);
    assert!((reconstructed.y - target.y).abs() < 1e-6);
}

#[test]
fn test_corexy_diagonal_motion() {
    let mut corexy = CoreXYKinematics;

    // Diagonal move: X=10, Y=10
    // Motor A = X + Y = 20
    // Motor B = X - Y = 0 (Motor B is stationary!)
    let target = Vector3::new(10.0, 10.0, 0.0);
    let motors = corexy.inverse_kinematics(target).expect("Valid IK target");

    assert_eq!(motors.x, 20.0, "Motor A should move double");
    assert_eq!(motors.y, 0.0, "Motor B should stay stationary during 45-deg diagonal");
}
