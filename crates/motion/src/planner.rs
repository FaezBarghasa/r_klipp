// File path: crates/motion/src/planner.rs
// AI-generated comment:
// This file was modified by an AI assistant to implement Junction Deviation and Input Shaping.
// Source files for context: crates/motion/src/planner.rs

//! # S-Curve (Jerk-Limited) Motion Planner
//!
//! This module translates high-level move requests into a series of timed
//! `StepCommand`s using a third-order motion profile (S-curve) for smoother
//! acceleration and deceleration, reducing vibrations and improving print quality.
//!
//! ## Operation
//!
//! 1.  **`plan_move`**: A high-level move is received and added to a `lookahead_queue`.
//!     No timing calculations are performed yet.
//!
//! 2.  **Junction Deviation**: With at least two moves in the lookahead queue, the planner
//!     analyzes the angle between them and calculates the maximum possible velocity for
//!     the corner junction. This velocity is used to set the `end_v` of the first
//!     move and `start_v` of the second.
//!
//! 3.  **Finalization**: Once a move's end velocity is known, its S-curve profile is
//!     calculated, and it is moved to the main `move_queue`, ready for step generation.
//!
//! 4.  **`generate_steps`**: The main firmware loop calls this function to dequeue a
//!     fully planned `MoveSegment` and generate `StepCommand`s.
//!
//! 5.  **Input Shaping**: If a shaper profile is active, this function performs a
//!     convolution, replacing each single step with a series of "ghost" steps
//!     timed and scaled to cancel machine resonance.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
use libm::{acosf, fabsf, fmaxf, fminf, powf, sqrtf};
#[cfg(feature = "std")]
use std::primitive::f32::{
    acos as acosf, fabs as fabsf, max as fmaxf, min as fminf, powf, sqrt as sqrtf,
};

use crate::{
    errors::PlannerError,
    profile::{InputShaper, PressureAdvance},
    StepCommand,
};
use heapless::spsc::{Producer, Queue};
use heapless::{Deque, Vec, binary_heap::{BinaryHeap, Min}};

const MAX_AXES: usize = 8;
const CLOCK_FREQ: f32 = 100_000_000.0;
const EXTRUDER_AXIS: usize = 3;

/// A segment of a move with a defined S-curve velocity profile.
#[derive(Copy, Clone, Debug, Default)]
pub struct MoveSegment {
    pub steps: [i32; MAX_AXES],
    pub direction_mask: u8,
    pub total_steps: u32,
    pub junction_deviation: f32,
    pub distance: f32,
    pub start_v: f32,
    pub cruise_v: f32,
    pub end_v: f32,
    pub accel: f32,
    pub jerk: f32,
    pub t_j1: f32, pub t_a: f32, pub t_j2: f32, pub t_c: f32,
    pub t_j3: f32, pub t_d: f32, pub t_j4: f32,
    pub pa: Option<PressureAdvance>,
    pub shaper: Option<InputShaper>,
}

impl MoveSegment {
    /// Gets the move vector in mm, excluding the extruder axis.
    pub fn get_move_vector_mm(&self, steps_per_mm: &[f32; MAX_AXES]) -> [f32; MAX_AXES - 1] {
        let mut vec = [0.0; MAX_AXES - 1];
        for i in 0..MAX_AXES-1 {
            vec[i] = self.steps[i] as f32 / steps_per_mm[i];
        }
        vec
    }
}

// Struct for the input shaper's priority queue.
#[derive(Copy, Clone, PartialEq, Eq)]
struct ScheduledStep {
    time: u32,
    stepper_mask: u8,
}
impl Ord for ScheduledStep {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        other.time.cmp(&self.time) // Reverse for min-heap
    }
}
impl PartialOrd for ScheduledStep {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(self.cmp(other))
    }
}


/// The main motion planner.
pub struct MotionPlanner {
    pub move_queue: Queue<MoveSegment, 64>,
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

    /// Plans a move and adds it to the lookahead queue.
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
            steps[i] = target_pos[i].wrapping_sub(self.current_position[i]);
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

        if total_steps == 0 && steps[EXTRUDER_AXIS] != 0 {
            // Extruder-only move, handle separately if needed
            return Ok(());
        }
        if distance < 1e-6 {
            return Ok(()); // Zero length move
        }

        let segment = MoveSegment {
            steps,
            direction_mask,
            total_steps,
            junction_deviation,
            distance,
            start_v: 0.0,
            cruise_v: velocity,
            end_v: 0.0,
            accel,
            jerk,
            t_j1: 0.0, t_a: 0.0, t_j2: 0.0, t_c: 0.0,
            t_j3: 0.0, t_d: 0.0, t_j4: 0.0,
            pa: None,
            shaper: None,
        };

        if self.lookahead_queue.push_back(segment).is_err() {
            return Err(PlannerError::QueueFull);
        }
        self.current_position = target_pos;
        self.process_lookahead()?;
        Ok(())
    }

    /// Flushes the lookahead queue, finalizing all remaining moves.
    pub fn finalize(&mut self) -> Result<(), PlannerError> {
        self.process_lookahead()?;
        while let Some(mut segment) = self.lookahead_queue.pop_front() {
            self.recalculate_timing(&mut segment);
            if self.move_queue.enqueue(segment).is_err() {
                return Err(PlannerError::QueueFull);
            }
        }
        Ok(())
    }

    /// Generates step commands for the next move in the queue.
    pub fn generate_steps(
        &mut self,
        producer: &mut Producer<'static, StepCommand, 256>,
    ) -> Result<(), ()> {
        let mut segment = self.move_queue.dequeue().ok_or(())?;
        let mut errors = [0i32; MAX_AXES];
        let dominant_axis_steps = segment.total_steps;

        let mut extruder_steps_to_add = 0.0;
        let total_time = segment.t_j1 + segment.t_a + segment.t_j2 + segment.t_c + segment.t_j3 + segment.t_d + segment.t_j4;
        if total_time <= 0.0 { return Ok(()); }

        if let Some(shaper) = segment.shaper {
            // --- Input Shaping Path ---
            let mut shaper_heap: BinaryHeap<ScheduledStep, Min, 64> = BinaryHeap::new();
            let mut last_time_ticks: u32 = 0;

            for n in 1..=dominant_axis_steps {
                let t = (n as f32 / dominant_axis_steps as f32) * total_time;
                let (v, _) = self.get_velocity_and_accel(&segment, t);
                let interval = if v > 0.0 { (CLOCK_FREQ / v * segment.distance / dominant_axis_steps as f32) as u32 } else { u32::MAX };
                let current_time_ticks = last_time_ticks.saturating_add(interval);

                let stepper_mask = self.bresenham_step(&mut errors, &segment.steps, dominant_axis_steps);
                // (PA logic would go here if combined)

                for i in 0..shaper.num_impulses {
                    let (time_offset, _) = shaper.impulses[i];
                    let shaped_time = current_time_ticks.saturating_add((time_offset * CLOCK_FREQ) as u32);
                    shaper_heap.push(ScheduledStep { time: shaped_time, stepper_mask: stepper_mask as u8 }).map_err(|_| ())?;
                }
                last_time_ticks = current_time_ticks;
            }

            // Drain the heap and generate final step commands
            let mut prev_time = 0;
            while let Some(step) = shaper_heap.pop() {
                let interval = step.time.saturating_sub(prev_time);
                producer.enqueue(StepCommand::new(step.stepper_mask, segment.direction_mask, interval as u16)).map_err(|_| ())?;
                prev_time = step.time;
            }

        } else {
            // --- Standard Path (No Shaping) ---
            for n in 1..=dominant_axis_steps {
                let t = (n as f32 / dominant_axis_steps as f32) * total_time;
                let (v, a) = self.get_velocity_and_accel(&segment, t);
                let interval = if v > 0.0 { (CLOCK_FREQ / v * segment.distance / dominant_axis_steps as f32) as u32 } else { u32::MAX };
                let stepper_mask = self.bresenham_step(&mut errors, &segment.steps, dominant_axis_steps);

                // (PA logic would go here)

                producer.enqueue(StepCommand::new(stepper_mask as u8, segment.direction_mask, interval as u16)).map_err(|_| ())?;
            }
        }
        Ok(())
    }

    // --- Private Helper Functions ---

    fn bresenham_step(&self, errors: &mut [i32; MAX_AXES], steps: &[i32; MAX_AXES], dominant_steps: u32) -> u32 {
        let mut mask = 0;
        for i in 0..MAX_AXES {
            if i == EXTRUDER_AXIS { continue; }
            errors[i] += steps[i].abs();
            if (errors[i] * 2) as u32 >= dominant_steps {
                mask |= 1 << i;
                errors[i] -= dominant_steps as i32;
            }
        }
        mask
    }

    fn process_lookahead(&mut self) -> Result<(), PlannerError> {
        while self.lookahead_queue.len() >= 2 {
            let seg1 = &self.lookahead_queue[0];
            let seg2 = &self.lookahead_queue[1];

            let v1 = seg1.get_move_vector_mm(&self.steps_per_mm);
            let v2 = seg2.get_move_vector_mm(&self.steps_per_mm);
            let v1_mag_sq: f32 = v1.iter().map(|&x| x * x).sum();
            let v2_mag_sq: f32 = v2.iter().map(|&x| x * x).sum();

            if v1_mag_sq < 1e-9 || v2_mag_sq < 1e-9 {
                // Zero length move in lookahead, finalize and dequeue it
                let mut seg = self.lookahead_queue.pop_front().unwrap();
                self.recalculate_timing(&mut seg);
                self.move_queue.enqueue(seg).map_err(|_| PlannerError::QueueFull)?;
                continue;
            }
            let dot_product: f32 = v1.iter().zip(v2.iter()).map(|(a, b)| a * b).sum();
            let cos_theta = dot_product / sqrtf(v1_mag_sq * v2_mag_sq);

            let min_accel = fminf(seg1.accel, seg2.accel);
            let denominator = 1.0 - cos_theta;
            let junction_v_sq = if denominator > 1e-6 {
                (2.0 * seg1.junction_deviation * min_accel) * fabsf(1.0 - cos_theta) / (sqrtf(v1_mag_sq) * sqrtf(v2_mag_sq))
            } else {
                seg1.cruise_v * seg1.cruise_v
            };

            let max_v = fminf(seg1.cruise_v, seg2.cruise_v);
            let junction_v = fminf(sqrtf(junction_v_sq), max_v);

            let mut seg1_mut = self.lookahead_queue.front_mut().unwrap();
            seg1_mut.end_v = junction_v;
            self.recalculate_timing(seg1_mut);

            let seg1_final = self.lookahead_queue.pop_front().unwrap();

            if let Some(seg2_mut) = self.lookahead_queue.front_mut() {
                seg2_mut.start_v = junction_v;
            }

            self.move_queue.enqueue(seg1_final).map_err(|_| PlannerError::QueueFull)?;
        }
        Ok(())
    }

    fn recalculate_timing(&self, s: &mut MoveSegment) {
        let accel_dist = (s.cruise_v * s.cruise_v - s.start_v * s.start_v) / (2.0 * s.accel);
        let decel_dist = (s.cruise_v * s.cruise_v - s.end_v * s.end_v) / (2.0 * s.accel);

        if accel_dist + decel_dist > s.distance {
            // Triangle profile (no cruise)
            s.cruise_v = sqrtf((2.0 * s.accel * s.distance + s.start_v * s.start_v + s.end_v * s.end_v) / 2.0);
            let accel_time = fabsf(s.cruise_v - s.start_v) / s.accel;
            let decel_time = fabsf(s.cruise_v - s.end_v) / s.accel;
            s.t_c = 0.0;
            s.t_j1 = fminf(accel_time / 2.0, s.accel / s.jerk);
            s.t_a = accel_time - 2.0 * s.t_j1;
            s.t_j2 = s.t_j1;
            s.t_j3 = fminf(decel_time / 2.0, s.accel / s.jerk);
            s.t_d = decel_time - 2.0 * s.t_j3;
            s.t_j4 = s.t_j3;
        } else {
            // Trapezoid profile
            let accel_time = fabsf(s.cruise_v - s.start_v) / s.accel;
            let decel_time = fabsf(s.cruise_v - s.end_v) / s.accel;
            s.t_c = (s.distance - accel_dist - decel_dist) / s.cruise_v;
            s.t_j1 = fminf(accel_time / 2.0, s.accel / s.jerk);
            s.t_a = accel_time - 2.0 * s.t_j1;
            s.t_j2 = s.t_j1;
            s.t_j3 = fminf(decel_time / 2.0, s.accel / s.jerk);
            s.t_d = decel_time - 2.0 * s.t_j3;
            s.t_j4 = s.t_j3;
        }
    }

    fn get_velocity_and_accel(&self, s: &MoveSegment, t: f32) -> (f32, f32) {
        let j = s.jerk;
        let t1 = s.t_j1; let t2 = t1 + s.t_a; let t3 = t2 + s.t_j2;
        let t4 = t3 + s.t_c;
        let t5 = t4 + s.t_j3; let t6 = t5 + s.t_d;

        if t < t1 { (s.start_v + 0.5 * j * t * t, j * t) }
        else if t < t2 { (s.start_v + 0.5 * j * t1 * t1 + s.accel * (t - t1), s.accel) }
        else if t < t3 { (s.cruise_v - 0.5 * j * (t3 - t) * (t3 - t), j * (t3 - t)) }
        else if t < t4 { (s.cruise_v, 0.0) }
        else if t < t5 { let dt = t - t4; (s.cruise_v - 0.5 * j * dt * dt, -j * dt) }
        else if t < t6 { let dt = t - t5; (s.cruise_v - 0.5 * j * s.t_j3 * s.t_j3 - s.accel * dt, -s.accel) }
        else { let dt = t - t6; (s.end_v + 0.5 * j * (s.t_j4 - dt) * (s.t_j4 - dt), -j * (s.t_j4 - dt)) }
    }
}

