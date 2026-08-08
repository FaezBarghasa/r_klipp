use heapless::spsc::Queue;

/// Kinematic boundaries for the G4 motion planning run
#[derive(Copy, Clone, Debug)]
pub struct KinematicLimits {
    pub max_velocity: f64,
    pub max_accel: f64,
    pub max_jerk: f64,
    pub max_snap: f64,
    pub max_crackle: f64,
}

/// Representation of a single continuous segment within the 31-Phase Generator
#[derive(Copy, Clone, Debug)]
pub struct TrajectoryPhase {
    pub duration: f64,
    pub crackle: f64,      // Constant crackle value during this phase
    pub snap_start: f64,   // Initial value of snap at entry
    pub jerk_start: f64,   // Initial jerk
    pub accel_start: f64,  // Initial acceleration
    pub vel_start: f64,    // Initial velocity
    pub pos_start: f64,    // Initial position
}

impl TrajectoryPhase {
    /// Computes analytical position, velocity, and acceleration at time delta_t
    /// within this phase using Taylor Series Integration.
    #[inline(always)]
    pub fn evaluate(&self, dt: f64) -> (f64, f64, f64) {
        let dt2 = dt * dt;
        let dt3 = dt2 * dt;
        let dt4 = dt3 * dt;
        let dt5 = dt4 * dt;

        let _snap = self.snap_start + self.crackle * dt;
        let _jerk = self.jerk_start + self.snap_start * dt + 0.5 * self.crackle * dt2;
        let accel = self.accel_start + self.jerk_start * dt + 0.5 * self.snap_start * dt2 + (1.0 / 6.0) * self.crackle * dt3;
        
        let vel = self.vel_start + self.accel_start * dt + 0.5 * self.jerk_start * dt2 
            + (1.0 / 6.0) * self.snap_start * dt3 + (1.0 / 24.0) * self.crackle * dt4;
            
        let pos = self.pos_start + self.vel_start * dt + 0.5 * self.accel_start * dt2 
            + (1.0 / 6.0) * self.jerk_start * dt3 + (1.0 / 24.0) * self.snap_start * dt4 + (1.0 / 120.0) * self.crackle * dt5;

        (pos, vel, accel)
    }
}

/// A lock-free planner queue for supplying the step generation interrupt 
/// with upcoming 31-phase trajectory segments.
pub struct G4TrajectoryQueue {
    // SPSC queue allows lock-free pushing from the planner and popping from the ISR
    phase_queue: Queue<TrajectoryPhase, 64>,
}

impl G4TrajectoryQueue {
    pub fn new() -> Self {
        Self {
            phase_queue: Queue::new(),
        }
    }

    pub fn push_phase(&mut self, phase: TrajectoryPhase) -> Result<(), TrajectoryPhase> {
        self.phase_queue.enqueue(phase)
    }

    pub fn pop_phase(&mut self) -> Option<TrajectoryPhase> {
        self.phase_queue.dequeue()
    }
}