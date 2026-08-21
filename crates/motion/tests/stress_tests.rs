//! Stress tests for the motion planner and input shaper.

use motion::trapezoidal::{TrapezoidalConstraints, TrapezoidalProfile};
use motion::lookahead::LookaheadQueue;
use motion::shaper::{InputShaper, ZVConfig};
use motion::g4_planner::{G4Profile, KinematicLimits};

#[test]
fn test_trapezoidal_stress_many_moves() {
    let mut queue: LookaheadQueue<64> = LookaheadQueue::new(0.05);

    let mut current = [0.0, 0.0, 0.0];
    for i in 1..=50 {
        let next = [i as f64 * 2.0, (i % 5) as f64 * 3.0, 0.0];
        let res = queue.push_move(current, next, 150.0, 3000.0);
        assert!(res.is_ok());
        current = next;
    }

    assert_eq!(queue.len(), 50);

    // Pop and verify each move is valid
    while let Some(plan) = queue.pop_move() {
        assert!(plan.total_time > 0.0);
        assert!(plan.distance > 0.0);
    }
}

#[test]
fn test_input_shaper_stress() {
    let cfg = ZVConfig {
        damping_ratio: 0.05,
        resonance_hz: 45.0,
    };
    let shaper = InputShaper::new(cfg);

    for raw_interval in (100..10_000).step_by(50) {
        let pulses = shaper.shape_step_timing(raw_interval);
        assert_eq!(pulses.len(), 2);
        let sum = pulses[0] + pulses[1];
        assert!((sum as i64 - raw_interval as i64).abs() <= 1);
    }
}

#[test]
fn test_g4_extreme_limits() {
    let limits = KinematicLimits {
        max_velocity: 500.0,
        max_accel: 10000.0,
        max_jerk: 100000.0,
        max_snap: 1000000.0,
        max_crackle: 10000000.0,
    };

    let profile = G4Profile::plan(0.0, 200.0, 0.0, 500.0, 0.0, limits);
    assert!(profile.total_duration > 0.0);

    // Sample across 100 points
    for i in 0..=100 {
        let t = (i as f64 / 100.0) * profile.total_duration;
        let (p, v, a) = profile.sample(t);
        assert!(!p.is_nan());
        assert!(!v.is_nan());
        assert!(!a.is_nan());
    }
}
