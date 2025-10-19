use compat_layer::migrator::migrate_config;
use compat_layer::models::*;
use std::fs;
use std::path::Path;

fn load_config(file_name: &str) -> String {
    let path = Path::new("tests/test_configs").join(file_name);
    fs::read_to_string(path).expect("Failed to read test config file")
}

#[test]
fn test_migrate_ender3_cartesian() {
    let content = load_config("ender3.cfg");
    let (config, report) = migrate_config(&content).unwrap();

    assert_eq!(config.kinematics, "cartesian");
    assert_eq!(config.max_velocity, 300.0);
    assert_eq!(config.max_accel, 3000.0);

    // Test Stepper X
    let stepper_x = config.steppers.x.unwrap();
    assert_eq!(stepper_x.stepper.step_pin, "ar54");
    assert_eq!(stepper_x.position_endstop, -5.0);
    assert_eq!(stepper_x.position_max, 235.0);

    // Test Extruder
    let extruder = config.extruder.unwrap();
    assert_eq!(extruder.sensor_type, "EPCOS 100K B57560G104F");
    assert_eq!(extruder.max_temp, 250.0);
    assert_eq!(extruder.stepper.unwrap().rotation_distance, 33.5);
    assert_eq!(extruder.nozzle_diameter.unwrap(), 0.4);

    // Test Heater Bed
    let heater_bed = config.heater_bed.unwrap();
    assert_eq!(heater_bed.heater_pin, "ar9");
    assert_eq!(heater_bed.max_temp, 130.0);

    // Check for unsupported sections
    assert!(report.unsupported_sections.contains(&"mcu".to_string()));
    assert!(report.unsupported_sections.contains(&"virtual_sdcard".to_string()));
    assert!(config.other_sections.contains_key("mcu"));
}

#[test]
fn test_migrate_corexy() {
    let content = load_config("corexy.cfg");
    let (config, _report) = migrate_config(&content).unwrap();

    assert_eq!(config.kinematics, "corexy");

    // CoreXY has stepper_x and stepper_y but they are for motors A and B
    // Our model maps them to x and y directly for simplicity.
    assert!(config.steppers.x.is_some());
    assert!(config.steppers.y.is_some());
    assert!(config.steppers.z.is_some());

    let stepper_y = config.steppers.y.unwrap();
    assert_eq!(stepper_y.stepper.dir_pin, "!P2.1");
    assert_eq!(stepper_y.stepper.rotation_distance, 40.0);
}

#[test]
fn test_migrate_skipr() {
    let content = load_config("skipr.cfg");
    let (config, report) = migrate_config(&content).unwrap();

    assert_eq!(config.kinematics, "cartesian");
    assert!(config.heater_bed.is_some());
    assert!(config.extruder.is_some());

    // Check that a section like [fan] is correctly identified as unsupported
    assert!(report.unsupported_sections.contains(&"fan".to_string()));
    assert!(config.other_sections.contains_key("fan"));
    assert_eq!(
        config.other_sections["fan"]["pin"],
        "PB15"
    );
}
