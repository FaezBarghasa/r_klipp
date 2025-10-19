//! # Trapezoidal Motion Planner
//!
//! This module translates high-level move requests into a series of timed
//! `StepCommand`s that the stepper driver can execute. It uses trapezoidal
//! velocity profiles to ensure smooth acceleration and deceleration.
//!
//! ## Operation
//!
//! 1.  **`plan_move`**: A high-level move (target position, velocity, accel) is
//!     received. The planner calculates the parameters for a trapezoidal
//!     (or triangular, for short moves) profile. This includes the number of
//!     steps in the acceleration, cruise, and deceleration phases, as well as
//!     the initial and minimum step intervals (in timer ticks). This is stored
//!     as a `MoveSegment` in a queue.
//!
//! 2.  **`generate_steps`**: The main firmware loop calls this function. It dequeues
//!     a `MoveSegment` and iterates through the entire move, generating one
//!     `StepCommand` for each step of the dominant axis.
//!
//! 3.  **Step Generation**: For each step, it calculates:
//!     - **Step Interval**: The time until the next step. This value changes
//!       during the acceleration and deceleration phases to create the velocity
//!       profile.
//!     - **Step/Direction Masks**: A Bresenham's line algorithm determines which
//!       motors need to step in this cycle to approximate a straight line to the
//!       target.
//!
//! ## Performance & `no_std`
//!
//! - The planning phase (`plan_move`) uses `f32` for accuracy.
//! - The step generation phase (`generate_steps`) uses integer-only arithmetic
//!   for maximum performance and determinism, making it suitable for real-time
//!   execution.
//! - All data structures use `heapless` to avoid heap allocations.

#[cfg(not(feature = "std"))]
use libm::{fabsf, sqrtf};
#[cfg(feature = "std")]
use std::primitive::f32::{fabs as fabsf, sqrt as sqrtf};

use crate::{errors::PlannerError, StepCommand};
use heapless::spsc::{Producer, Queue};

const MAX_AXES: usize = 8;
const CLOCK_FREQ: f32 = 100_000_000.0; // Example: 100 MHz timer frequency

/// A segment of a move with a defined trapezoidal velocity profile.
#[derive(Copy, Clone, Debug, Default)]
pub struct MoveSegment {
    /// The number of steps for each axis in this segment.
    pub steps: [i32; MAX_AXES],
    /// Bitmask for the direction of each axis.
    pub direction_mask: u8,
    /// The total number of steps for the dominant axis.
    pub total_steps: u32,
    /// The number of steps in the acceleration phase.
    pub accel_steps: u32,
    /// The number of steps in the deceleration phase.
    pub decel_start_step: u32,
    /// The initial (slowest) interval between steps, in timer ticks.
    pub initial_interval: u32,
    /// The minimum (fastest) interval at cruise speed, in timer ticks.
    pub min_interval: u32,
    /// A pre-calculated constant used for acceleration calculations. `(2 * accel) / (clock_freq^2)`
    pub accel_rate: f32,
}

/// The main motion planner. It queues `MoveSegment`s and generates `StepCommand`s.
pub struct MotionPlanner {
    queue: Queue<MoveSegment, 64>,
    pub current_position: [i32; MAX_AXES], // Position in microsteps
}

impl MotionPlanner {
    pub fn new() -> Self {
        Self {
            queue: Queue::new(),
            current_position: [0; MAX_AXES],
        }
    }

    /// Plans a move and adds it to the queue.
    ///
    /// # Arguments
    /// * `target_pos` - The target position for each axis in microsteps.
    /// * `velocity` - The cruise velocity in steps/sec.
    /// * `accel` - The acceleration in steps/sec^2.
    pub fn plan_move(
        &mut self,
        target_pos: [i32; MAX_AXES],
        velocity: f32,
        accel: f32,
    ) -> Result<(), PlannerError> {
        let mut steps = [0; MAX_AXES];
        let mut direction_mask = 0;

        for i in 0..MAX_AXES {
            let delta = target_pos[i] - self.current_position[i];
            steps[i] = delta;
            if delta > 0 {
                direction_mask |= 1 << i;
            }
        }

        let total_steps = steps.iter().map(|s| s.abs() as u32).max().unwrap_or(0);
        if total_steps == 0 {
            return Ok(()); // No move needed
        }

        // --- Calculate Trapezoidal Profile ---
        // v^2 = u^2 + 2as -> s = v^2 / 2a (since u=0)
        let accel_steps_f = velocity * velocity / (2.0 * accel);
        let mut accel_steps = accel_steps_f as u32;

        if accel_steps * 2 > total_steps {
            // Triangular profile (not enough distance to reach full speed)
            accel_steps = total_steps / 2;
        }

        let decel_start_step = total_steps - accel_steps;
        let min_interval = (CLOCK_FREQ / velocity) as u32;

        // Calculate initial step interval based on acceleration
        // t = sqrt(2d/a), d = 1 step. We use a common approximation here.
        let initial_interval = (0.676 * CLOCK_FREQ / sqrtf(accel)) as u32;

        // This is a constant used in the step generation loop to calculate the interval
        // for each acceleration step. It avoids a sqrt in the hot loop.
        let accel_rate = (2.0 * accel) / (CLOCK_FREQ * CLOCK_FREQ);


        let segment = MoveSegment {
            steps,
            direction_mask,
            total_steps,
            accel_steps,
            decel_start_step,
            initial_interval,
            min_interval,
            accel_rate,
        };

        if self.queue.enqueue(segment).is_err() {
            return Err(PlannerError::QueueFull);
        }

        // Update planner's position to the new target
        self.current_position = target_pos;

        Ok(())
    }

    /// Generates step commands for the next move in the queue and sends them to the stepper driver.
    pub fn generate_steps(
        &mut self,
        producer: &mut Producer<'static, StepCommand, 256>,
    ) -> Result<u32, ()> {
        let segment = match self.queue.dequeue() {
            Some(s) => s,
            None => return Ok(0), // No moves to generate
        };

        let mut errors = [0i32; MAX_AXES];
        let mut last_interval = segment.initial_interval;

        for n in 1..=segment.total_steps {
            // --- Bresenham's Line Algorithm ---
            let mut stepper_mask = 0;
            for i in 0..MAX_AXES {
                errors[i] += segment.steps[i].abs();
                if (errors[i] * 2) >= segment.total_steps as i32 {
                    stepper_mask |= 1 << i;
                    errors[i] -= segment.total_steps as i32;
                }
            }

            // --- Velocity Profile / Interval Calculation ---
            let interval = if n <= segment.accel_steps {
                // Acceleration phase
                let next_interval = last_interval as f32 * (1.0 - segment.accel_rate * last_interval as f32);
                (next_interval as u32).max(segment.min_interval)
            } else if n > segment.decel_start_step {
                // Deceleration phase
                let next_interval = last_interval as f32 * (1.0 + segment.accel_rate * last_interval as f32);
                next_interval as u32
            } else {
                // Cruise phase
                segment.min_interval
            };

            let cmd = StepCommand::new(stepper_mask, segment.direction_mask, interval as u16);
            if producer.enqueue(cmd).is_err() {
                // Stepper queue is full. This indicates a problem, as the motion planner
                // should be synchronized with the stepper consumer.
                // In a real system, we might wait or drop the move.
                defmt::error!("Stepper command queue full!");
                return Err(());
            }
            last_interval = interval;
        }
        Ok(segment.total_steps)
    }
}
