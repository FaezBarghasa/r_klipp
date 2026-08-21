//! Tests for the motion planner's determinism and correctness.

use motion::trapezoidal::{TrapezoidalConstraints, TrapezoidalProfile};
use motion::lookahead::LookaheadQueue;
use motion::planner::{plan_segment, MotionConstraints};
use nalgebra::Vector3;

#[test]
fn test_planner_generates_deterministic_trapezoid_move() {
    let cons = TrapezoidalConstraints {
        v_max: 100.0,
        a_max: 1000.0,
        v_entry: 0.0,
        v_exit: 0.0,
    };

    let p1 = TrapezoidalProfile::plan(50.0, cons).unwrap();
    let p2 = TrapezoidalProfile::plan(50.0, cons).unwrap();

    assert_eq!(p1, p2);
    assert_eq!(p1.v_cruise, 100.0);

    let ticks1 = p1.compute_step_intervals(80.0, 1_000_000.0);
    let ticks2 = p2.compute_step_intervals(80.0, 1_000_000.0);
    assert_eq!(ticks1, ticks2);
}

#[test]
fn test_scurve_determinism() {
    let start = Vector3::new(0.0, 0.0, 0.0);
    let end = Vector3::new(100.0, 50.0, 0.0);
    let cons = MotionConstraints {
        vmax: 200.0,
        amax: 2000.0,
        jmax: 20000.0,
    };

    let s1 = plan_segment(start, end, cons).unwrap();
    let s2 = plan_segment(start, end, cons).unwrap();
    assert_eq!(s1, s2);
}

#[test]
fn test_lookahead_queue_determinism() {
    let mut q1: LookaheadQueue<16> = LookaheadQueue::new(0.05);
    let mut q2: LookaheadQueue<16> = LookaheadQueue::new(0.05);

    q1.push_move([0.0, 0.0, 0.0], [50.0, 0.0, 0.0], 80.0, 1000.0).unwrap();
    q1.push_move([50.0, 0.0, 0.0], [50.0, 50.0, 0.0], 80.0, 1000.0).unwrap();

    q2.push_move([0.0, 0.0, 0.0], [50.0, 0.0, 0.0], 80.0, 1000.0).unwrap();
    q2.push_move([50.0, 0.0, 0.0], [50.0, 50.0, 0.0], 80.0, 1000.0).unwrap();

    let m1 = q1.pop_move().unwrap();
    let m2 = q2.pop_move().unwrap();
    assert_eq!(m1, m2);
}
