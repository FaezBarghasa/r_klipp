//! Planner actor for direct spline interpolation.
#![no_std]

use crate::motion::kinematics::splines::math::{ControlPoint, SplineError};
use crate::motion::kinematics::splines::parameterization::Spline;
use heapless::spsc::{Consumer, Producer, Queue};
use heapless::Vec;

/// The micro-step distance for sampling the spline.
const SAMPLING_DISTANCE_MM: f32 = 0.01;

/// Maximum number of points to buffer from the spline sampling.
const POINT_BUFFER_SIZE: usize = 512;

/// Actor that receives spline definitions, samples them at a high frequency,
/// and feeds the resulting points into a kinematics queue.
pub struct SplinePlanner<'a> {
    spline_consumer: Consumer<'a, Spline, 16>,
    point_producer: Producer<'a, ControlPoint, POINT_BUFFER_SIZE>,
    current_spline: Option<Spline>,
    samples: Vec<ControlPoint, POINT_BUFFER_SIZE>,
    sample_index: usize,
}

impl<'a> SplinePlanner<'a> {
    pub fn new(
        spline_consumer: Consumer<'a, Spline, 16>,
        point_producer: Producer<'a, ControlPoint, POINT_BUFFER_SIZE>,
    ) -> Self {
        Self {
            spline_consumer,
            point_producer,
            current_spline: None,
            samples: Vec::new(),
            sample_index: 0,
        }
    }

    /// Runs the planner's main loop. This should be called periodically.
    pub fn run(&mut self) -> Result<(), SplineError> {
        // If there's no active spline, try to get one.
        if self.current_spline.is_none() {
            if let Some(spline) = self.spline_consumer.dequeue() {
                self.samples.clear();
                spline.sample_by_distance(SAMPLING_DISTANCE_MM, &mut self.samples)?;
                self.sample_index = 0;
                self.current_spline = Some(spline);
            }
        }

        // If we have an active spline, process its samples.
        if self.current_spline.is_some() {
            while !self.point_producer.is_full() {
                if let Some(point) = self.samples.get(self.sample_index) {
                    self.point_producer.enqueue(*point).ok();
                    self.sample_index += 1;
                } else {
                    // We've exhausted all samples for the current spline.
                    self.current_spline = None;
                    break; // Exit the loop and wait for the next spline.
                }
            }
        }

        Ok(())
    }
}

// In a real actor model, this would be more complex, likely involving async/await.
// For this `#![no_std]` context, we simulate with queues and a `run` method.

#[cfg(test)]
mod tests {
    use super::*;
    use heapless::spsc::Queue;

    fn create_test_spline() -> Spline {
        let degree = 2;
        let knots: &[f32] = &[0.0, 0.0, 0.0, 1.0, 1.0, 1.0];
        let control_points: &[ControlPoint] = &[
            ControlPoint::new(0.0, 0.0, 0.0),
            ControlPoint::new(5.0, 10.0, 0.0),
            ControlPoint::new(10.0, 0.0, 0.0),
        ];
        Spline::new(degree, knots, control_points).unwrap()
    }

    #[test]
    fn test_planner_receives_spline_and_produces_points() {
        let mut spline_q: Queue<Spline, 16> = Queue::new();
        let mut point_q: Queue<ControlPoint, POINT_BUFFER_SIZE> = Queue::new();

        let (mut spline_p, spline_c) = spline_q.split();
        let (point_p, mut point_c) = point_q.split();

        let mut planner = SplinePlanner::new(spline_c, point_p);

        // Initially, no points should be produced.
        planner.run().unwrap();
        assert_eq!(point_c.dequeue(), None);

        // Enqueue a spline.
        let spline = create_test_spline();
        let expected_total_points = (spline.total_arc_length() / SAMPLING_DISTANCE_MM).floor() as usize + 1;
        spline_p.enqueue(spline).unwrap();

        // Run the planner. It should consume the spline and start producing points.
        planner.run().unwrap();

        // Check that some points have been produced.
        let mut points_produced = 0;
        while let Some(_) = point_c.dequeue() {
            points_produced += 1;
        }
        assert!(points_produced > 0);
        assert!(points_produced <= POINT_BUFFER_SIZE);

        // Run the planner until all points are produced.
        while planner.current_spline.is_some() {
            // Consume all points to make space in the queue
            while let Some(_) = point_c.dequeue() {}
            planner.run().unwrap();
             while let Some(_) = point_c.dequeue() {
                points_produced += 1;
            }
        }

        // The total number of points should match the expected sampling count.
        assert_eq!(points_produced, expected_total_points);

        // After finishing, the planner should be idle again.
        planner.run().unwrap();
        assert_eq!(point_c.dequeue(), None);
    }

    #[test]
    fn test_planner_handles_multiple_splines_sequentially() {
        let mut spline_q: Queue<Spline, 16> = Queue::new();
        let mut point_q: Queue<ControlPoint, POINT_BUFFER_SIZE> = Queue::new();

        let (mut spline_p, spline_c) = spline_q.split();
        let (point_p, mut point_c) = point_q.split();

        let mut planner = SplinePlanner::new(spline_c, point_p);

        // Enqueue two splines.
        let spline1 = create_test_spline();
        let spline2 = create_test_spline();
        let expected_points1 = (spline1.total_arc_length() / SAMPLING_DISTANCE_MM).floor() as usize + 1;
        let expected_points2 = (spline2.total_arc_length() / SAMPLING_DISTANCE_MM).floor() as usize + 1;

        spline_p.enqueue(spline1).unwrap();
        spline_p.enqueue(spline2).unwrap();

        let mut total_points = 0;

        // Process the first spline completely.
        while planner.current_spline.is_none() || planner.sample_index < planner.samples.len() {
             while let Some(_) = point_c.dequeue() {} // Clear queue
             planner.run().unwrap();
             while let Some(_) = point_c.dequeue() {
                total_points += 1;
             }
             if planner.current_spline.is_none() && !spline_p.ready() { break; }
        }

        assert_eq!(total_points, expected_points1);
        assert!(planner.current_spline.is_none());

        // Now run again to process the second spline.
        total_points = 0;
        while planner.current_spline.is_none() || planner.sample_index < planner.samples.len() {
             while let Some(_) = point_c.dequeue() {} // Clear queue
             planner.run().unwrap();
             while let Some(_) = point_c.dequeue() {
                total_points += 1;
             }
             if planner.current_spline.is_none() { break; }
        }

        assert_eq!(total_points, expected_points2);
    }
}
