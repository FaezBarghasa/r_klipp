use thermal::mpc::MpcThermalEngine;
use thermal::safety::RunawayWatchdogConfig;

#[test]
fn test_mpc_engine_converges_to_250c() {
    let watchdog_config = RunawayWatchdogConfig {
        max_temp_c: 300.0,
        min_temp_c: -10.0,
        max_heating_rate_c_per_s: 15.0,
        runaway_timeout_ms: 10_000,
        hysteresis_temp_c: 5.0,
    };


    let mut engine = MpcThermalEngine::new(
        (0.95, 0.05, 0.0, 0.90), // A matrix
        (0.1, 0.02),             // B matrix
        (0.01, 0.0, 0.0, 0.0),   // G matrix
        (1.0, 0.0, 0.0, 1.0),    // P initial covariance
        (0.01, 0.01),            // Q process noise
        0.05,                    // R measurement noise
        25.0,                    // initial x0 (ambient)
        0.0,                     // initial x1
        250.0,                   // Target temperature 250.0 °C
        0.05,                    // Kp
        0.01,                    // Feedforward loss
        watchdog_config,
        0,                       // initial timestamp ms
    );

    let mut current_temp = 25.0f32;
    let mut timestamp = 0u32;

    // Simulate 500 control cycles (100ms per cycle = 50s)
    for _ in 0..500 {
        timestamp += 100;
        let power = engine.step(current_temp, 25.0, 0.0, timestamp);

        // Simulated physical thermal plant: heating rate proportional to power - heat dissipation
        let heat_loss = 0.01 * (current_temp - 25.0);
        let heat_gain = power * 6.0;
        current_temp += (heat_gain - heat_loss) * 0.1;
    }

    // After 50 seconds, the temperature should have ramped significantly toward target
    assert!(current_temp > 200.0, "Thermal plant should have heated up toward 250C, got {}", current_temp);
}

#[test]
fn test_mpc_engine_estop_cuts_heater_power() {
    let watchdog_config = RunawayWatchdogConfig {
        max_temp_c: 280.0,
        min_temp_c: 0.0,
        max_heating_rate_c_per_s: 15.0,
        runaway_timeout_ms: 5000,
        hysteresis_temp_c: 5.0,
    };


    let mut engine = MpcThermalEngine::new(
        (0.98, 0.02, 0.0, 0.95),
        (0.15, 0.05),
        (0.0, 0.0, 0.0, 0.0),
        (1.0, 0.0, 0.0, 1.0),
        (0.01, 0.01),
        0.05,
        25.0,
        0.0,
        200.0,
        0.1,
        0.0,
        watchdog_config,
        0,
    );

    // Initial step: full power requested
    let initial_power = engine.step(25.0, 25.0, 0.0, 100);
    assert!(initial_power > 0.0, "Initial heating power should be active");

    // Over-temperature triggers emergency stop
    let mut power = initial_power;
    for t in (200..=1000).step_by(100) {
        power = engine.step(350.0, 25.0, 0.0, t);
        if power == 0.0 {
            break;
        }
    }
    assert_eq!(power, 0.0, "Over-temperature must trigger watchdog and cut power to 0.0");

}

