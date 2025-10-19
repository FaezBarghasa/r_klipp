//! Tests for the motion planner's determinism and correctness.

use motion::planner::MotionPlanner;
use motion::StepCommand;
use heapless::spsc::Queue;

#[test]
fn test_planner_generates_correct_trapezoid_move() {
    let mut planner = MotionPlanner::new();
    static mut STEP_QUEUE: Queue<StepCommand, 256> = Queue::new();
    let (mut producer, mut consumer) = unsafe { STEP_QUEUE.split() };

    let mut target = [0; 8];
    target[0] = 800; // 10mm at 80 steps/mm
    target[1] = 800;

    // Plan a 10mm x 10mm diagonal move
    // Velocity: 800 steps/sec, Accel: 1000 steps/sec^2
    planner.plan_move(target, 800.0, 1000.0).unwrap();

    // Generate the steps into the queue
    let steps_generated = planner.generate_steps(&mut producer).unwrap();

    // Dominant axis is diagonal, total steps is sqrt(800^2 + 800^2) ~ 1131, but planner
    // uses the max of cartesian axes, so total_steps = 800.
    assert_eq!(steps_generated, 800);
    assert_eq!(consumer.len(), 800);

    // --- Verify the "Golden" Trace of the first few steps ---
    // These values depend on CLOCK_FREQ and the accel calculations.
    // They serve as a regression test to ensure the output is deterministic.

    // 1. First step should have both motors on, and a long initial interval.
    let s1 = consumer.dequeue().unwrap();
    assert_eq!(s1.stepper_mask, 0b0000_0011); // X and Y
    assert_eq!(s1.direction_mask, 0b0000_0011); // Both forward
    assert!(s1.interval_ticks > 20000); // Should be slow initially

    // 2. Second step interval should be shorter (accelerating)
    let s2 = consumer.dequeue().unwrap();
    assert!(s2.interval_ticks < s1.interval_ticks);

    // 3. Drain the queue and find the cruise phase
    let mut min_interval = s1.interval_ticks;
    for _ in 2..steps_generated {
        let s = consumer.dequeue().unwrap();
        if s.interval_ticks < min_interval {
            min_interval = s.interval_ticks;
        }
    }

    // The minimum interval should correspond to the cruise velocity
    // Expected: CLOCK_FREQ / velocity = 100_000_000 / 800 = 125_000
    // This will not be exact due to the nature of the iterative calculation, but should be close.
    assert!((min_interval as i32 - 125_000).abs() < 500);

}
