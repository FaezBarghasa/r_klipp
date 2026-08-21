//! Look-Ahead Queue with Junction Deviation cornering optimization.

use heapless::Vec;
use crate::trapezoidal::{TrapezoidalConstraints, TrapezoidalProfile};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MoveBlock {
    pub start: [f64; 3],
    pub end: [f64; 3],
    pub v_requested: f64,
    pub a_max: f64,
    pub v_entry: f64,
    pub v_exit: f64,
    pub distance: f64,
}

impl MoveBlock {
    pub fn new(start: [f64; 3], end: [f64; 3], v_requested: f64, a_max: f64) -> Self {
        let dx = end[0] - start[0];
        let dy = end[1] - start[1];
        let dz = end[2] - start[2];
        let distance = (dx * dx + dy * dy + dz * dz).sqrt();

        Self {
            start,
            end,
            v_requested,
            a_max,
            v_entry: 0.0,
            v_exit: 0.0,
            distance,
        }
    }

    /// Unit direction vector
    pub fn unit_vector(&self) -> [f64; 3] {
        if self.distance < 1e-9 {
            [0.0, 0.0, 0.0]
        } else {
            [
                (self.end[0] - self.start[0]) / self.distance,
                (self.end[1] - self.start[1]) / self.distance,
                (self.end[2] - self.start[2]) / self.distance,
            ]
        }
    }
}

pub struct LookaheadQueue<const N: usize> {
    pub blocks: Vec<MoveBlock, N>,
    pub junction_deviation: f64,
}

impl<const N: usize> LookaheadQueue<N> {
    pub fn new(junction_deviation: f64) -> Self {
        Self {
            blocks: Vec::new(),
            junction_deviation: junction_deviation.max(0.01),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.blocks.is_empty()
    }

    pub fn is_full(&self) -> bool {
        self.blocks.is_full()
    }

    pub fn len(&self) -> usize {
        self.blocks.len()
    }

    /// Adds a new move to the lookahead buffer and recomputes optimal junction velocities.
    pub fn push_move(&mut self, start: [f64; 3], end: [f64; 3], v_req: f64, a_max: f64) -> Result<(), ()> {
        let block = MoveBlock::new(start, end, v_req, a_max);
        if block.distance < 1e-9 {
            return Ok(()); // Ignore zero-length moves
        }

        if self.blocks.is_full() {
            return Err(());
        }

        self.blocks.push(block).map_err(|_| ())?;
        self.recalculate();
        Ok(())
    }

    /// Pops the fully planned front move from the lookahead buffer.
    pub fn pop_move(&mut self) -> Option<TrapezoidalProfile> {
        if self.blocks.is_empty() {
            return None;
        }

        let block = self.blocks.remove(0);
        let cons = TrapezoidalConstraints {
            v_max: block.v_requested,
            a_max: block.a_max,
            v_entry: block.v_entry,
            v_exit: block.v_exit,
        };

        TrapezoidalProfile::plan(block.distance, cons).ok()
    }

    /// Calculates junction velocity between block `prev` and block `next`.
    fn compute_junction_velocity(prev: &MoveBlock, next: &MoveBlock, junction_deviation: f64) -> f64 {
        let u1 = prev.unit_vector();
        let u2 = next.unit_vector();

        // Cosine of angle between direction vectors
        let cos_theta = -(u1[0] * u2[0] + u1[1] * u2[1] + u1[2] * u2[2]);
        if cos_theta > 0.999999 {
            // Reversal of direction: full stop required
            return 0.0;
        }
        if cos_theta < -0.999999 {
            // Colinear straight line: no slowdown required
            return prev.v_requested.min(next.v_requested);
        }

        // Half angle formula: sin(theta / 2) = sqrt((1 + cos_theta) / 2)
        let sin_half_theta = ((1.0 + cos_theta) / 2.0).sqrt();
        let r = (junction_deviation * sin_half_theta) / (1.0 - sin_half_theta);

        let a_max = prev.a_max.min(next.a_max);
        let v_corner = (a_max * r).sqrt();

        v_corner.min(prev.v_requested).min(next.v_requested)
    }

    /// Forward and backward planning pass over lookahead ring buffer.
    pub fn recalculate(&mut self) {
        let n = self.blocks.len();
        if n == 0 {
            return;
        }

        // Compute junction speed limits
        for i in 0..n - 1 {
            let v_j = Self::compute_junction_velocity(&self.blocks[i], &self.blocks[i + 1], self.junction_deviation);
            self.blocks[i].v_exit = v_j;
            self.blocks[i + 1].v_entry = v_j;
        }

        // The very last block must decelerate to 0 exit velocity
        self.blocks[n - 1].v_exit = 0.0;

        // Backward pass: ensure each block can decelerate in time for exit velocity
        for i in (0..n).rev() {
            let max_v_entry_sq = self.blocks[i].v_exit * self.blocks[i].v_exit + 2.0 * self.blocks[i].a_max * self.blocks[i].distance;
            let max_v_entry = max_v_entry_sq.sqrt().min(self.blocks[i].v_requested);
            if self.blocks[i].v_entry > max_v_entry {
                self.blocks[i].v_entry = max_v_entry;
                if i > 0 {
                    self.blocks[i - 1].v_exit = max_v_entry;
                }
            }
        }

        // Forward pass: ensure each block can accelerate from entry velocity
        for i in 0..n {
            let max_v_exit_sq = self.blocks[i].v_entry * self.blocks[i].v_entry + 2.0 * self.blocks[i].a_max * self.blocks[i].distance;
            let max_v_exit = max_v_exit_sq.sqrt().min(self.blocks[i].v_requested);
            if self.blocks[i].v_exit > max_v_exit {
                self.blocks[i].v_exit = max_v_exit;
                if i + 1 < n {
                    self.blocks[i + 1].v_entry = max_v_exit;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lookahead_queue_cornering() {
        let mut queue: LookaheadQueue<16> = LookaheadQueue::new(0.05);

        // Right angle turn: (0,0) -> (100,0) -> (100,100)
        queue.push_move([0.0, 0.0, 0.0], [100.0, 0.0, 0.0], 100.0, 1000.0).unwrap();
        queue.push_move([100.0, 0.0, 0.0], [100.0, 100.0, 0.0], 100.0, 1000.0).unwrap();

        assert_eq!(queue.len(), 2);
        // Entry of move 1 should be 0.0
        assert_eq!(queue.blocks[0].v_entry, 0.0);
        // Junction between move 1 and move 2 should be > 0 (doesn't need to stop completely)
        assert!(queue.blocks[0].v_exit > 0.0);
        assert_eq!(queue.blocks[0].v_exit, queue.blocks[1].v_entry);
        // Exit of final move should be 0.0
        assert_eq!(queue.blocks[1].v_exit, 0.0);

        let plan1 = queue.pop_move().unwrap();
        assert!(plan1.v_exit > 0.0);
    }
}
