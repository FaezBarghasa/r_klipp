#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KinematicError {
    LimitExceeded,
    TrajectoryPlanningFailed,
    OvershootDetected,
}