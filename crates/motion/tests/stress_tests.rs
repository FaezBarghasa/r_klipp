// File path: crates/motion/tests/stress_tests.rs
#![cfg(test)]

use motion::{
    errors::PlannerError,
    planner::MotionPlanner,
    profile::{InputShaper, ShaperType},
    StepCommand,
};
use heapless::spsc::Queue;

const DUMMY_STEPS_PER_MM: [f32; 8] = [80.0, 80.0, 400.0, 500.0, 0.0, 0.0, 0.0, 0.0];

#[test]
fn test_queue_full_error() {
    let mut planner = MotionPlanner::new(DUMMY_STEPS_PER_MM);
    let mut target_pos = [0; 8];

    // Rapidly plan more moves than the queue can hold (size 64)
    for i in 1..=100 {
        target_pos[0] = i * 100;
        let result = planner.plan_move(target_pos, 10000.0, 5000.0, 0.0, 0.01);

        if i > 64 {
            // After 64 moves, we expect the queue to be full
            assert_eq!(result, Err(PlannerError::QueueFull));
        } else {
            assert!(result.is_ok());
        }
    }
}

#[test]
fn test_high_step_rate_simulation() {
    static mut COMMAND_QUEUE: Queue<StepCommand, 1024> = Queue::new();
    let (mut producer, mut consumer) = unsafe { COMMAND_QUEUE.split() };

    let mut planner = MotionPlanner::new(DUMMY_STEPS_PER_MM);
    let mut target_pos = [0; 8];
    target_pos[0] = 200 * 80; // 200mm move

    // Plan a move with very high velocity and acceleration
    // Velocity: 200,000 steps/sec (2500 mm/s @ 80 steps/mm)
    // Accel: 5,000,000 steps/sec^2
    planner.plan_move(target_pos, 2500.0, 62500.0, 200000.0, 0.01).unwrap();

    planner.generate_steps(&mut producer).unwrap();

    let mut last_interval = u16::MAX;
    let mut cruise_intervals = 0;
    let mut min_interval = u16::MAX;

    while let Some(cmd) = consumer.dequeue() {
        // Interval must never be zero, as this would be an infinite step rate
        assert_ne!(cmd.interval_ticks, 0, "Step interval must not be zero");

        if cmd.interval_ticks < min_interval {
            min_interval = cmd.interval_ticks;
        }

        // Check that intervals are monotonically decreasing during acceleration
        if cruise_intervals == 0 {
            if cmd.interval_ticks > last_interval {
                cruise_intervals += 1; // Reached cruise or decel
            }
            assert!(cmd.interval_ticks <= last_interval, "Interval should decrease during accel");
        }

        last_interval = cmd.interval_ticks;
    }

    // Expected cruise interval = CLOCK_FREQ / (vel * steps/mm) is not straightforward with S-curve.
    // However, we can check that the minimum interval is very small.
    // For 200,000 steps/sec, interval should be around 100,000,000 / 200,000 = 500 ticks.
    assert!(min_interval < 600, "Minimum interval was {}, expected < 600 for high speed", min_interval);
}

#[test]
fn test_vincent_many_small_segments() {
    static mut COMMAND_QUEUE: Queue<StepCommand, 8192> = Queue::new();
    let (mut producer, mut consumer) = unsafe { COMMAND_QUEUE.split() };

    let mut planner = MotionPlanner::new(DUMMY_STEPS_PER_MM);
    let segments = 200;
    let radius = 50.0 * 80.0; // 50mm radius in steps

    let mut last_interval = 0;
    let mut stops = 0;

    // Simulate printing a circle with many small segments
    for i in 1..=segments {
        let angle = (i as f32 / segments as f32) * 2.0 * core::f32::consts::PI;
        let mut target_pos = planner.current_position;
        target_pos[0] = (radius * libm::cosf(angle)) as i32;
        target_pos[1] = (radius * libm::sinf(angle)) as i32;

        // Use a high junction deviation to encourage cornering speed
        planner.plan_move(target_pos, 100.0, 2000.0, 10000.0, 0.5).unwrap();
        planner.generate_steps(&mut producer).unwrap();

        let first_step = consumer.dequeue().unwrap();

        // Check if the planner came to a near-stop.
        // A high initial interval indicates a near-zero starting velocity.
        // With lookahead, this should be low for all but the first segment.
        if i > 1 && first_step.interval_ticks > 10000 {
            stops += 1;
        }

        while let Some(cmd) = consumer.dequeue() {
            last_interval = cmd.interval_ticks;
        }
    }

    // With proper junction deviation handling, there should be no complete stops between segments.
    assert!(stops < 5, "Planner stopped {} times between small segments, expected < 5", stops);
}

