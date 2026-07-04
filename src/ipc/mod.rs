//! Inter-Task Communication (IPC) queues for r_klipp.
//! This module provides lock-free queues for passing data between asynchronous
//! tasks and hard real-time interrupt handlers.
//! This file corresponds to Task 1.2 of the development plan.

#![no_std]

use heapless::spsc::{Queue, Producer, Consumer};
use portable_atomic::{AtomicBool, Ordering};

// Re-export for convenience in other modules
pub use heapless::spsc::Error;

/// A command to be executed by the step generator ISR.
/// For now, this is a placeholder. In a real system, it would contain
/// precise timing and axis information for a single step pulse.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct StepCommand {
    /// The number of timer ticks to wait before executing this step.
    pub interval: u32,
    /// A bitmask of axes to step.
    pub direction_mask: u32,
    /// A bitmask of directions for each axis.
    pub step_mask: u32,
}

/// Telemetry data sent from the ISR back to the async world.
/// This could include information about the step execution, encoder feedback, etc.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TelemetryPacket {
    /// The timestamp of the event, in timer ticks.
    pub timestamp: u64,
    /// A bitmask indicating which axes have completed their moves.
    pub completed_mask: u32,
}

/// A lock-free, single-producer, single-consumer queue for `StepCommand`s.
/// This is used to send commands from the async Planner task to the Step Generator ISR.
pub struct StepCommandQueue<const N: usize> {
    queue: Queue<StepCommand, N>,
}

impl<const N: usize> StepCommandQueue<N> {
    /// Creates a new, empty `StepCommandQueue`.
    /// This function is `const`, so it can be used to initialize a `static` variable.
    pub const fn new() -> Self {
        Self {
            queue: Queue::new(),
        }
    }

    /// Splits the queue into a producer and a consumer half.
    pub fn split<'a>(&'a mut self) -> (Producer<'a, StepCommand, N>, Consumer<'a, StepCommand, N>) {
        self.queue.split()
    }
}

/// A lock-free, single-producer, single-consumer queue for `TelemetryPacket`s.
/// This is used to send data from the ISR back to the async Telemetry task.
pub struct TelemetryQueue<const N: usize> {
    queue: Queue<TelemetryPacket, N>,
}

impl<const N: usize> TelemetryQueue<N> {
    /// Creates a new, empty `TelemetryQueue`.
    /// This function is `const`, so it can be used to initialize a `static` variable.
    pub const fn new() -> Self {
        Self {
            queue: Queue::new(),
        }
    }

    /// Splits the queue into a producer and a consumer half.
    pub fn split<'a>(&'a mut self) -> (Producer<'a, TelemetryPacket, N>, Consumer<'a, TelemetryPacket, N>) {
        self.queue.split()
    }
}

// Example of how to create and use these queues in a static context.
// This would typically be in your `main.rs`.
#[cfg(test)]
mod tests {
    use super::*;
    use static_cell::StaticCell;

    const QUEUE_SIZE: usize = 8;

    #[test]
    fn test_step_command_queue_split_and_use() {
        static STEP_QUEUE_CELL: StaticCell<StepCommandQueue<QUEUE_SIZE>> = StaticCell::new();
        let queue = STEP_QUEUE_CELL.init(StepCommandQueue::<QUEUE_SIZE>::new());
        let (mut producer, mut consumer) = queue.split();

        let cmd = StepCommand { interval: 100, direction_mask: 1, step_mask: 1 };
        assert!(producer.enqueue(cmd).is_ok());

        let received_cmd = consumer.dequeue();
        assert_eq!(received_cmd, Some(cmd));
        assert!(consumer.dequeue().is_none());
    }

    #[test]
    fn test_telemetry_queue_split_and_use() {
        static TELEMETRY_QUEUE_CELL: StaticCell<TelemetryQueue<QUEUE_SIZE>> = StaticCell::new();
        let queue = TELEMETRY_QUEUE_CELL.init(TelemetryQueue::<QUEUE_SIZE>::new());
        let (mut producer, mut consumer) = queue.split();

        let packet = TelemetryPacket { timestamp: 12345, completed_mask: 0xFF };
        assert!(producer.enqueue(packet).is_ok());

        let received_packet = consumer.dequeue();
        assert_eq!(received_packet, Some(packet));
        assert!(consumer.dequeue().is_none());
    }

    #[test]
    fn test_queue_full_and_empty() {
        static QUEUE_CELL: StaticCell<StepCommandQueue<QUEUE_SIZE>> = StaticCell::new();
        let queue = QUEUE_CELL.init(StepCommandQueue::<QUEUE_SIZE>::new());
        let (mut producer, mut consumer) = queue.split();

        for i in 0..QUEUE_SIZE {
            let cmd = StepCommand { interval: i as u32, direction_mask: 1, step_mask: 1 };
            assert!(producer.enqueue(cmd).is_ok());
        }

        // Queue is now full
        let cmd_overflow = StepCommand { interval: 99, direction_mask: 1, step_mask: 1 };
        assert_eq!(producer.enqueue(cmd_overflow), Err(Error::Full(cmd_overflow)));

        for i in 0..QUEUE_SIZE {
            let received = consumer.dequeue();
            assert!(received.is_some());
            assert_eq!(received.unwrap().interval, i as u32);
        }

        // Queue is now empty
        assert!(consumer.dequeue().is_none());
    }
}
