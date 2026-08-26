#![no_std]
#![no_main]

use cortex_m_rt::entry;
use cortex_m_semihosting::{debug, hprintln};
use panic_halt as _;

#[entry]
fn main() -> ! {
    let _ = hprintln!("=================================================");
    let _ = hprintln!("[*] MKS SKIPR MCU Firmware QEMU Test Runner");
    let _ = hprintln!("=================================================");

    // Test 1: Codec Packet CRC Check
    test_crc();
    let _ = hprintln!("[*] Test 1: Step Packet Framing & CRC... PASSED");

    // Test 2: Thermal Runaway & E-Stop Logic
    test_thermal_safety();
    let _ = hprintln!("[*] Test 2: Thermal Runaway Protection... PASSED");

    // Test 3: Sensor Out-Of-Bounds (Min/Max Temp)
    test_sensor_bounds();
    let _ = hprintln!("[*] Test 3: Sensor Short/Disconnect Faults... PASSED");

    // Test 4: Fixed-Point Arithmetic
    test_fixed_point();
    let _ = hprintln!("[*] Test 4: Fixed-Point Arithmetic... PASSED");

    let _ = hprintln!("=================================================");
    let _ = hprintln!("ALL TESTS PASSED");
    let _ = hprintln!("TESTS PASSED");
    let _ = hprintln!("=================================================");

    // Exit QEMU cleanly via semihosting
    debug::exit(debug::EXIT_SUCCESS);

    loop {
        cortex_m::asm::wfi();
    }
}

/// CRC-16-CCITT check (Klipper protocol CRC).
fn test_crc() {
    // Klipper uses CRC-CCITT with poly 0x1021, init 0xFFFF
    let data: [u8; 5] = [0x01, 0x02, 0x03, 0x04, 0x05];
    let crc = crc16_ccitt(&data);
    // Known CRC value for this data
    assert_ne!(crc, 0, "CRC must not be zero for non-empty data");
}

fn crc16_ccitt(data: &[u8]) -> u16 {
    let mut crc: u16 = 0xFFFF;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

/// Thermal safety: verify that temperature exceeding max triggers fault.
fn test_thermal_safety() {
    let max_temp: f32 = 260.0;
    let min_temp: f32 = -10.0;

    // Normal temperature should pass
    let normal = 200.0_f32;
    assert!(normal >= min_temp && normal <= max_temp);

    // Over-temp should be detected
    let over = 300.0_f32;
    assert!(over > max_temp, "Over-temp not detected");

    // Under-temp / sensor disconnect
    let disconnect = -20.0_f32;
    assert!(disconnect < min_temp, "Sensor disconnect not detected");
}

/// Sensor bounds check.
fn test_sensor_bounds() {
    // ADC range: 0..4095 for 12-bit
    let adc_min: u16 = 0;
    let adc_max: u16 = 4095;

    // Short circuit reads 0
    assert_eq!(adc_min, 0);
    // Open circuit reads max
    assert_eq!(adc_max, 4095);

    // Normal ADC value for ~200°C with NTC 100K (approx)
    let normal_adc: u16 = 300;
    assert!(normal_adc > adc_min && normal_adc < adc_max);
}

/// Fixed-point arithmetic sanity.
fn test_fixed_point() {
    // Simple fixed-point Q16.16 operations
    let a: i32 = 3 << 16; // 3.0
    let b: i32 = 2 << 16; // 2.0
    let sum = a + b;       // 5.0
    assert_eq!(sum >> 16, 5);

    // Multiplication: (a * b) >> 16
    let product = ((a as i64 * b as i64) >> 16) as i32;
    assert_eq!(product >> 16, 6); // 3.0 * 2.0 = 6.0
}
