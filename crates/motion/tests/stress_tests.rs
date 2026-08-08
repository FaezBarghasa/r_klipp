// File path: crates/motion/tests/stress_tests.rs
// AI-generated comment:
// This file was modified by an AI assistant to add tests for Junction Deviation and Input Shaping.
// Source files for context: crates/motion/tests/stress_tests.rs, crates/motion/src/planner.rs

#![cfg(test)]

use motion::{
    error::PlannerError,
    planner::MotionPlanner,
    profile::{InputShaper, ShaperType},
};
use mcu_drivers::stepper::StepSegment;
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
        if i > 64 {
            assert_eq!(result, Err(PlannerError::QueueFull));
        } else {
            assert!(result.is_ok());
        }
    }
}

#[test]
fn test_junction_deviation_cornering_speed() {
    let mut planner = MotionPlanner::new(DUMMY_STEPS_PER_MM);
    static mut Q_CORNER: Queue<StepSegment, 1024> = Queue::new();
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
    let _steps_move1 = consumer.len();
    while consumer.len() > 0 { consumer.dequeue(); } // Drain

    planner.generate_steps(&mut producer).unwrap();
    let first_step_move2 = consumer.dequeue().unwrap();

    // Now, create a reference move that starts from standstill
    let mut planner_standstill = MotionPlanner::new(DUMMY_STEPS_PER_MM);
    static mut Q_SS: Queue<StepSegment, 1024> = Queue::new();
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
    static mut COMMAND_QUEUE: Queue<StepSegment, 2048> = Queue::new();
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

    let mut step_times = std::vec::Vec::new();
    let mut total_time_ticks: u32 = 0;
    while let Some(cmd) = consumer.dequeue() {
        total_time_ticks = total_time_ticks.saturating_add(cmd.interval_ticks as u32);
        step_times.push(total_time_ticks);
    }

    // Verify chronological order
    for i in 1..step_times.len() {
        assert!(step_times[i] >= step_times[i-1]);
    }

    // Verify ZV shaping: for each of the first 10 steps, there should be a corresponding
    // step at exactly t + half_period_ticks.
    for i in 0..10 {
        let t = step_times[i];
        let target_t = t + half_period_ticks;
        let found = step_times.iter().any(|&x| (x as i32 - target_t as i32).abs() < 5);
        assert!(found, "Could not find second impulse for step {} at time {} (expected {})", i, t, target_t);
    }
}
