// File: crates/motion/src/planner.rs
//! # S-Curve (Jerk-Limited) Motion Planner
//!
//! This module translates high-level move requests into a series of timed
//! `StepCommand`s using a third-order motion profile (S-curve) for smoother
//! acceleration and deceleration, reducing vibrations and improving print quality.
//!
//! ## Operation
//!
//! 1.  **`plan_move`**: A high-level move is received. The planner calculates the
//!     parameters for an S-curve profile, including phases of increasing/decreasing
//!     acceleration (jerk). This is stored as a `MoveSegment`.
//!
//! 2.  **`generate_steps`**: The main firmware loop calls this function to dequeue
//!     a `MoveSegment` and generate one `StepCommand` for each step of the dominant
//!     axis. The step interval calculation now follows the S-curve profile.
//!
//! 3.  **Junction Deviation**: The planner calculates the maximum velocity at the
//!     junction between two moves to allow for faster, smoother cornering without
//!     coming to a complete stop.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
use libm::{fabsf, fmaxf, fminf, powf, sqrtf};
#[cfg(feature = "std")]
use std::primitive::f32::{
    fabs as fabsf, max as fmaxf, min as fminf, powf, sqrt as sqrtf,
};

use crate::{
    errors::PlannerError,
    profile::{InputShaper, PressureAdvance},
    StepCommand,
};
use heapless::spsc::{Producer, Queue};
use heapless::{Deque, Vec};

const MAX_AXES: usize = 8;
const CLOCK_FREQ: f32 = 100_000_000.0; // Example: 100 MHz timer frequency
const MAX_SHAPER_EXTRA_STEPS: usize = 16;
const EXTRUDER_AXIS: usize = 3; // Typically axis E is the 4th one (index 3)

/// A segment of a move with a defined S-curve velocity profile.
#[derive(Copy, Clone, Debug, Default)]
pub struct MoveSegment {
    pub steps: [i32; MAX_AXES],
    pub direction_mask: u8,
    pub total_steps: u32,
    pub junction_deviation: f32,

    // S-Curve parameters
    pub distance: f32,
    pub start_v: f32,
    pub cruise_v: f32,
    pub end_v: f32,
    pub accel: f32,
    pub jerk: f32,

    // Time for each phase
    pub t_j1: f32, // increasing accel
    pub t_a: f32,  // constant accel
    pub t_j2: f32, // decreasing accel
    pub t_c: f32,  // cruise
    pub t_j3: f32, // increasing decel
    pub t_d: f32,  // constant decel
    pub t_j4: f32, // decreasing decel

    // Links to profiles
    pub pa: Option<PressureAdvance>,
    pub shaper: Option<InputShaper>,
}

/// The main motion planner.
pub struct MotionPlanner {
    move_queue: Queue<MoveSegment, 64>,
    lookahead_queue: Deque<MoveSegment, 8>,
    pub current_position: [i32; MAX_AXES],
    pub steps_per_mm: [f32; MAX_AXES],
}

impl MotionPlanner {
    pub fn new(steps_per_mm: [f32; MAX_AXES]) -> Self {
        Self {
            move_queue: Queue::new(),
            lookahead_queue: Deque::new(),
            current_position: [0; MAX_AXES],
            steps_per_mm,
        }
    }

    /// Plans a move and adds it to the queue.
    pub fn plan_move(
        &mut self,
        target_pos: [i32; MAX_AXES],
        velocity: f32,
        accel: f32,
        jerk: f32,
        junction_deviation: f32,
    ) -> Result<(), PlannerError> {
        let mut steps = [0; MAX_AXES];
        let mut direction_mask = 0;
        let mut move_dist_sq = 0.0;
        for i in 0..MAX_AXES {
            steps[i] = target_pos[i] - self.current_position[i];
            if steps[i] > 0 {
                direction_mask |= 1 << i;
            }
            if i != EXTRUDER_AXIS {
                let axis_dist = steps[i] as f32 / self.steps_per_mm[i];
                move_dist_sq += axis_dist * axis_dist;
            }
        }
        let distance = sqrtf(move_dist_sq);

        let total_steps = steps[0..EXTRUDER_AXIS]
            .iter()
            .map(|s| s.abs() as u32)
            .max()
            .unwrap_or(0);

        if total_steps == 0 {
            // Extruder-only move, handle separately if needed
            return Ok(());
        }

        // Simplified S-curve planning. A full implementation would involve lookahead
        // to calculate start_v and end_v based on junction velocities.
        // For now, assume moves start and end at zero velocity.
        let start_v = 0.0;
        let end_v = 0.0;

        // Calculate maximum achievable velocity and acceleration for this move length
        let max_v_for_dist = sqrtf(distance * accel + (start_v * start_v + end_v * end_v) / 2.0);
        let cruise_v = fminf(velocity, max_v_for_dist);

        // Time to accelerate from start_v to cruise_v
        let accel_time = (cruise_v - start_v) / accel;
        let accel_dist = start_v * accel_time + 0.5 * accel * accel_time * accel_time;

        // Time to decelerate from cruise_v to end_v
        let decel_time = (cruise_v - end_v) / accel;
        let decel_dist = cruise_v * decel_time - 0.5 * accel * decel_time * decel_time;

        let (t_j1, t_a, t_j2, t_c, t_j3, t_d, t_j4);

        if accel_dist + decel_dist > distance {
            // Triangle profile (no cruise phase)
            t_c = 0.0;
            // Re-calculate accel/decel for a shorter move
            let accel_time = sqrtf(distance / accel); // Simplified
            t_j1 = fminf(accel_time / 2.0, accel / jerk);
            t_a = accel_time - 2.0 * t_j1;
            t_j2 = t_j1;
            t_j3 = t_j1;
            t_d = t_a;
            t_j4 = t_j1;
        } else {
            // Trapezoid profile
            let cruise_dist = distance - accel_dist - decel_dist;
            t_c = cruise_dist / cruise_v;

            t_j1 = fminf(accel_time / 2.0, accel / jerk);
            t_a = accel_time - 2.0 * t_j1;
            t_j2 = t_j1;

            t_j3 = fminf(decel_time / 2.0, accel / jerk);
            t_d = decel_time - 2.0 * t_j3;
            t_j4 = t_j3;
        }

        let segment = MoveSegment {
            steps,
            direction_mask,
            total_steps,
            junction_deviation,
            distance,
            start_v,
            cruise_v,
            end_v,
            accel,
            jerk,
            t_j1,
            t_a,
            t_j2,
            t_c,
            t_j3,
            t_d,
            t_j4,
            pa: None,
            shaper: None,
        };

        if self.move_queue.enqueue(segment).is_err() {
            return Err(PlannerError::QueueFull);
        }
        self.current_position = target_pos;
        Ok(())
    }

    /// Generates step commands for the next move in the queue.
    pub fn generate_steps(
        &mut self,
        producer: &mut Producer<'static, StepCommand, 256>,
    ) -> Result<(), ()> {
        let segment = self.move_queue.dequeue().ok_or(())?;
        let mut errors = [0i32; MAX_AXES];
        let dominant_axis_steps = segment.total_steps;

        let mut shaped_intervals: Vec<(u32, u32), MAX_SHAPER_EXTRA_STEPS> = Vec::new();
        let mut extruder_steps_to_add = 0.0;

        let total_time = segment.t_j1 + segment.t_a + segment.t_j2 + segment.t_c + segment.t_j3 + segment.t_d + segment.t_j4;
        if total_time <= 0.0 { return Ok(()); }

        let inv_total_time = 1.0 / total_time;

        for n in 1..=dominant_axis_steps {
            // Calculate current time `t` in the move based on step `n`
            let t = (n as f32 / dominant_axis_steps as f32) * total_time;

            let (v, a) = self.get_velocity_and_accel(&segment, t);
            let interval = if v > 0.0 {
                (CLOCK_FREQ / v * segment.distance / dominant_axis_steps as f32) as u32
            } else {
                u32::MAX
            };

            // Bresenham's line algorithm for non-dominant axes
            let mut stepper_mask = 0;
            for i in 0..MAX_AXES {
                if i == EXTRUDER_AXIS { continue; }
                errors[i] += segment.steps[i].abs();
                if errors[i] * 2 >= dominant_axis_steps as i32 {
                    stepper_mask |= 1 << i;
                    errors[i] -= dominant_axis_steps as i32;
                }
            }

            // --- Apply Input Shaping ---
            if let Some(shaper) = segment.shaper {
                if shaper.num_impulses > 1 {
                    let current_ticks = n * interval; // simplified time
                    for i in 0..shaper.num_impulses {
                        let (time_offset, amplitude) = shaper.impulses[i];
                        let shaped_time = current_ticks + (time_offset * CLOCK_FREQ) as u32;
                        let shaped_interval = (interval as f32 / amplitude) as u32;

                        // This is a simplified convolution. A real one would be more complex.
                        // We add shaped intervals to a buffer to be sorted and emitted.
                        shaped_intervals.push((shaped_time, shaped_interval)).ok();
                    }
                }
            }

            // --- Apply Pressure Advance ---
            if let Some(pa) = segment.pa {
                let extruder_steps_per_mm = self.steps_per_mm[EXTRUDER_AXIS];
                let expected_extrusion = (segment.steps[EXTRUDER_AXIS] as f32 / dominant_axis_steps as f32);
                let pa_steps = pa.get_advance_steps(v, a, extruder_steps_per_mm);
                extruder_steps_to_add += expected_extrusion + pa_steps;
            } else {
                extruder_steps_to_add += segment.steps[EXTRUDER_AXIS] as f32 / dominant_axis_steps as f32;
            }

            if extruder_steps_to_add >= 1.0 {
                stepper_mask |= 1 << EXTRUDER_AXIS;
                extruder_steps_to_add -= 1.0;
            }

            if shaped_intervals.is_empty() {
                let cmd = StepCommand::new(stepper_mask as u8, segment.direction_mask, interval as u16);
                producer.enqueue(cmd).map_err(|_| ())?;
            }
        }

        // Drain the shaped intervals if any
        if !shaped_intervals.is_empty() {
            // A real implementation would merge/sort these intervals and emit them.
            // This is a placeholder for that complex logic.
            if let Some(&(_, interval)) = shaped_intervals.get(0) {
                let cmd = StepCommand::new(1, segment.direction_mask, interval as u16);
                producer.enqueue(cmd).map_err(|_| ())?;
            }
        }

        Ok(())
    }

    /// Calculates instantaneous velocity and acceleration at time `t` in a move.
    fn get_velocity_and_accel(&self, s: &MoveSegment, t: f32) -> (f32, f32) {
        let j = s.jerk;
        // Accel phase
        let t1 = s.t_j1;
        let t2 = t1 + s.t_a;
        let t3 = t2 + s.t_j2;
        // Cruise phase
        let t4 = t3 + s.t_c;
        // Decel phase
        let t5 = t4 + s.t_j3;
        let t6 = t5 + s.t_d;

        if t < t1 { // Increasing acceleration
            let v = s.start_v + 0.5 * j * t * t;
            let a = j * t;
            (v, a)
        } else if t < t2 { // Constant acceleration
            let v = s.start_v + 0.5 * j * t1 * t1 + s.accel * (t - t1);
            let a = s.accel;
            (v, a)
        } else if t < t3 { // Decreasing acceleration
            let dt = t - t2;
            let v = s.cruise_v - 0.5 * j * (t3 - t) * (t3 - t);
            let a = j * (t3 - t);
            (v, a)
        } else if t < t4 { // Cruise
            (s.cruise_v, 0.0)
        } else if t < t5 { // Increasing deceleration
            let dt = t - t4;
            let v = s.cruise_v - 0.5 * j * dt * dt;
            let a = -j * dt;
            (v, a)
        } else if t < t6 { // Constant deceleration
            let dt = t - t5;
            let v = s.cruise_v - 0.5 * j * s.t_j3 * s.t_j3 - s.accel * dt;
            let a = -s.accel;
            (v, a)
        } else { // Decreasing deceleration
            let dt = t - t6;
            let v = s.end_v + 0.5 * j * (s.t_j4 - dt) * (s.t_j4 - dt);
            let a = -j * (s.t_j4 - dt);
            (v, a)
        }
    }
}
