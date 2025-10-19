//! Error types for the motion planning crate.

/// Represents errors that can occur during motion planning.
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PlannerError {
    /// The motion planning queue is full and cannot accept new moves.
    QueueFull,
    /// The requested move is impossible (e.g., zero distance).
    InvalidMove,
}
