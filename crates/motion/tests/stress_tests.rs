// File path: crates/motion/tests/stress_tests.rs
// AI-generated comment:
// This file was modified by an AI assistant to add tests for Junction Deviation and Input Shaping.
// Source files for context: crates/motion/tests/stress_tests.rs, crates/motion/src/planner.rs

#![cfg(test)]

use motion::{
    errors::PlannerError,
    planner::{MoveSegment, MotionPlanner},
    profile::{InputShaper, ShaperType},
    StepCommand,
};
use heapless::spsc::Queue;

const DUMMY_STEPS_PER_MM: [f32; 8] = [80.0, 80.0, 400.0, 500.0, 0.0, 0.0, 0.0, 0.0];
const CLOCK_FREQ: f32 = 100_000_000.0;

#[test]
fn test_queue_full_error() {
    let mut planner = MotionPlanner::new(DUMMY_STEPS_PER_MM);
    let mut target_pos = [0; 8];

    for i in 1..=100 {
        target_pos[0] = (i * 100) as i32;
        let result = planner.plan_move(target_pos, 100.0, 5000.0, 10000.0, 0.01);

        // The lookahead queue holds 8, the move queue holds 64.
        // We will hit an error when both are getting full.
        if i > 64 + 8 {
            assert_eq!(result, Err(PlannerError::QueueFull));
        } else {
            assert!(result.is_ok());
        }
    }
}

#[test]
fn test_junction_deviation_cornering_speed() {
    let mut planner = MotionPlanner::new(DUMMY_STEPS_PER_MM);
    static mut Q_CORNER: Queue<StepCommand, 1024> = Queue::new();
    let (mut producer, mut consumer) = unsafe { Q_CORNER.split() };

    // Move 1: 10mm along X
    let mut pos1 = planner.current_position;
    pos1[0] = (10 * 80) as i32;
    planner.plan_move(pos1, 100.0, 2000.0, 10000.0, 0.05).unwrap();

    // Move 2: 10mm along Y (90-degree corner)
    let mut pos2 = pos1;
    pos2[1] = (10 * 80) as i32;
    planner.plan_move(pos2, 100.0, 2000.0, 10000.0, 0.05).unwrap();

    // Finalize to process all segments
    planner.finalize().unwrap();

    // Generate steps for both moves
    planner.generate_steps(&mut producer).unwrap();
    let steps_move1 = consumer.len();
    while consumer.len() > 0 { consumer.dequeue(); } // Drain

    planner.generate_steps(&mut producer).unwrap();
    let first_step_move2 = consumer.dequeue().unwrap();

    // Now, create a reference move that starts from standstill
    let mut planner_standstill = MotionPlanner::new(DUMMY_STEPS_PER_MM);
    static mut Q_SS: Queue<StepCommand, 1024> = Queue::new();
    let (mut producer_ss, mut consumer_ss) = unsafe { Q_SS.split() };

    // Plan only the second move, so it must start from v=0
    planner_standstill.current_position = pos1;
    planner_standstill.plan_move(pos2, 100.0, 2000.0, 10000.0, 0.05).unwrap();
    planner_standstill.finalize().unwrap();
    planner_standstill.generate_steps(&mut producer_ss).unwrap();
    let first_step_standstill = consumer_ss.dequeue().unwrap();

    // With junction deviation, the second move should not start from zero velocity.
    // A higher start velocity means it reaches its first step faster, resulting
    // in a smaller interval for the first step command.
    assert!(
        first_step_move2.interval_ticks < first_step_standstill.interval_ticks,
        "Junction move interval ({}) should be smaller than standstill interval ({})",
        first_step_move2.interval_ticks, first_step_standstill.interval_ticks
    );
}

#[test]
fn test_input_shaping_zv_application() {
    static mut COMMAND_QUEUE: Queue<StepCommand, 2048> = Queue::new();
    let (mut producer, mut consumer) = unsafe { COMMAND_QUEUE.split() };

    let mut planner = MotionPlanner::new(DUMMY_STEPS_PER_MM);
    let mut target_pos = [0; 8];
    target_pos[0] = (10 * 80) as i32; // 10mm move = 800 steps

    // Plan a move with a very high acceleration to force a long cruise phase
    planner.plan_move(target_pos, 100.0, 50000.0, 1e9, 0.0).unwrap();
    planner.finalize().unwrap();

    // Modify the generated segment to apply input shaping
    let mut move_seg = planner.move_queue.dequeue().unwrap();
    let shaper = InputShaper::new(ShaperType::ZV, 40.0, 0.1);
    move_seg.shaper = Some(shaper);
    planner.move_queue.enqueue(move_seg).unwrap();

    // Generate steps
    planner.generate_steps(&mut producer).unwrap();

    // --- Verification ---
    let num_original_steps = 10 * 80;
    let num_impulses = 2; // For ZV
    let num_generated_steps = consumer.len();
    assert_eq!(num_generated_steps, num_original_steps * num_impulses);

    let shaper_period_s = 1.0 / 40.0;
    let half_period_ticks = (0.5 * shaper_period_s * CLOCK_FREQ) as u32;

    let mut step_times = heapless::Vec::<u32, 20>::new();
    let mut total_time_ticks: u32 = 0;

    while let Some(cmd) = consumer.dequeue() {
        if step_times.len() >= 20 { break; }
        total_time_ticks = total_time_ticks.saturating_add(cmd.interval_ticks as u32);
        step_times.push(total_time_ticks).unwrap();
    }

    // The time delta between the two impulses for a single original step should be half a shaper period.
    let delta1 = step_times[1].saturating_sub(step_times[0]);
    assert!((delta1 as i32 - half_period_ticks as i32).abs() < 2, "Impulse delta was {}, expected {}", delta1, half_period_ticks);

    let delta2 = step_times[3].saturating_sub(step_times[2]);
    assert!((delta2 as i32 - half_period_ticks as i32).abs() < 2, "Impulse delta was {}, expected {}", delta2, half_period_ticks);

    // The time between the first impulse of one step and the first impulse of the next step
    // should be the original (unshaped) step interval.
    let original_interval = (CLOCK_FREQ / (100.0 * 80.0)) as u32; // CLOCK_FREQ / (vel_mm_s * steps_per_mm)
    let delta_orig = step_times[2].saturating_sub(step_times[0]);
    assert!((delta_orig as i32 - original_interval as i32).abs() < 2, "Original interval delta was {}, expected {}", delta_orig, original_interval);
}

