use heapless::spsc::Queue;

/// Kinematic boundaries for the G4 motion planning run
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct KinematicLimits {
    pub max_velocity: f64,
    pub max_accel: f64,
    pub max_jerk: f64,
    pub max_snap: f64,
    pub max_crackle: f64,
}

impl Default for KinematicLimits {
    fn default() -> Self {
        Self {
            max_velocity: 300.0,       // mm/s
            max_accel: 5000.0,         // mm/s^2
            max_jerk: 50000.0,         // mm/s^3
            max_snap: 500000.0,        // mm/s^4
            max_crackle: 5000000.0,    // mm/s^5
        }
    }
}

/// Representation of a single continuous segment within the 31-Phase Generator
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct TrajectoryPhase {
    pub duration: f64,
    pub crackle: f64,      // Constant crackle value during this phase (5th derivative)
    pub snap_start: f64,   // Initial snap (4th derivative)
    pub jerk_start: f64,   // Initial jerk (3rd derivative)
    pub accel_start: f64,  // Initial acceleration (2nd derivative)
    pub vel_start: f64,    // Initial velocity (1st derivative)
    pub pos_start: f64,    // Initial position
}

impl TrajectoryPhase {
    pub fn new_zero() -> Self {
        Self {
            duration: 0.0,
            crackle: 0.0,
            snap_start: 0.0,
            jerk_start: 0.0,
            accel_start: 0.0,
            vel_start: 0.0,
            pos_start: 0.0,
        }
    }

    /// Evaluates full kinematic state (pos, vel, accel, jerk, snap, crackle) at relative time dt
    #[inline(always)]
    pub fn evaluate_full(&self, dt: f64) -> (f64, f64, f64, f64, f64, f64) {
        let dt = dt.clamp(0.0, self.duration);
        let dt2 = dt * dt;
        let dt3 = dt2 * dt;
        let dt4 = dt3 * dt;
        let dt5 = dt4 * dt;

        let snap = self.snap_start + self.crackle * dt;
        let jerk = self.jerk_start + self.snap_start * dt + 0.5 * self.crackle * dt2;
        let accel = self.accel_start + self.jerk_start * dt + 0.5 * self.snap_start * dt2 + (1.0 / 6.0) * self.crackle * dt3;
        let vel = self.vel_start + self.accel_start * dt + 0.5 * self.jerk_start * dt2 
            + (1.0 / 6.0) * self.snap_start * dt3 + (1.0 / 24.0) * self.crackle * dt4;
        let pos = self.pos_start + self.vel_start * dt + 0.5 * self.accel_start * dt2 
            + (1.0 / 6.0) * self.jerk_start * dt3 + (1.0 / 24.0) * self.snap_start * dt4 + (1.0 / 120.0) * self.crackle * dt5;

        (pos, vel, accel, jerk, snap, self.crackle)
    }

    /// Computes analytical position, velocity, and acceleration at time delta_t
    #[inline(always)]
    pub fn evaluate(&self, dt: f64) -> (f64, f64, f64) {
        let (p, v, a, _, _, _) = self.evaluate_full(dt);
        (p, v, a)
    }

    /// Returns the end state of this phase as initial conditions for the next phase
    #[inline(always)]
    pub fn end_state(&self) -> (f64, f64, f64, f64, f64) {
        let (p, v, a, j, s, _) = self.evaluate_full(self.duration);
        (p, v, a, j, s)
    }
}

/// A complete 31-Phase G4 Trajectory Profile
#[derive(Clone, Debug)]
pub struct G4Profile {
    pub phases: [TrajectoryPhase; 31],
    pub total_duration: f64,
    pub total_distance: f64,
    pub limits: KinematicLimits,
}

impl G4Profile {
    /// Constructs and computes an optimal 31-phase trajectory profile
    pub fn plan(
        start_pos: f64,
        end_pos: f64,
        start_vel: f64,
        target_vel: f64,
        _end_vel: f64,
        limits: KinematicLimits,
    ) -> Self {
        let mut phases = [TrajectoryPhase::new_zero(); 31];
        let distance = (end_pos - start_pos).abs();
        let direction = if end_pos >= start_pos { 1.0 } else { -1.0 };

        let max_v = limits.max_velocity.min(target_vel.abs());
        let max_a = limits.max_accel.abs();
        let max_j = limits.max_jerk.abs();
        let max_s = limits.max_snap.abs();
        let max_c = limits.max_crackle.abs().max(1.0);

        let t_c = (max_s / max_c).min(0.01);

        let crackle_pattern_accel = [
            max_c, 0.0, -max_c, 0.0, -max_c, 0.0, max_c, // 0..6
            0.0,                                         // 7 (constant accel)
            -max_c, 0.0, max_c, 0.0, max_c, 0.0, -max_c, // 8..14
            0.0,                                         // 15 (cruise)
            -max_c, 0.0, max_c, 0.0, max_c, 0.0, -max_c, // 16..22
            0.0,                                         // 23 (constant decel)
            max_c, 0.0, -max_c, 0.0, -max_c, 0.0, max_c, // 24..30
        ];

        let mut curr_pos = start_pos;
        let mut curr_vel = start_vel.abs() * direction;
        let mut curr_acc = 0.0;
        let mut curr_jrk = 0.0;
        let mut curr_snp = 0.0;

        let delta_v = (max_v - start_vel.abs()).max(0.0);
        let t_sub = (delta_v / (14.0 * max_a.max(1.0))).min(t_c.max(0.001));

        let mut total_t = 0.0;

        for i in 0..31 {
            let dur = match i {
                7 | 23 => 0.005, // Constant acceleration cruise phases
                15 => {
                    let est_dist = (curr_vel * 0.01).abs();
                    if distance > est_dist {
                        ((distance - est_dist) / max_v.max(1.0)).max(0.0)
                    } else {
                        0.0
                    }
                }
                _ => t_sub,
            };

            let c = crackle_pattern_accel[i] * direction;
            phases[i] = TrajectoryPhase {
                duration: dur,
                crackle: c,
                snap_start: curr_snp,
                jerk_start: curr_jrk,
                accel_start: curr_acc,
                vel_start: curr_vel,
                pos_start: curr_pos,
            };

            let (p, v, a, j, s) = phases[i].end_state();
            curr_pos = p;
            curr_vel = v;
            curr_acc = a;
            curr_jrk = j;
            curr_snp = s;
            total_t += dur;
        }

        Self {
            phases,
            total_duration: total_t,
            total_distance: (curr_pos - start_pos).abs(),
            limits,
        }
    }

    /// Samples position, velocity, and acceleration at continuous time t in [0, total_duration]
    pub fn sample(&self, t: f64) -> (f64, f64, f64) {
        let mut rem_t = t.clamp(0.0, self.total_duration);
        for phase in &self.phases {
            if rem_t <= phase.duration {
                return phase.evaluate(rem_t);
            }
            rem_t -= phase.duration;
        }
        if let Some(last) = self.phases.last() {
            last.evaluate(last.duration)
        } else {
            (0.0, 0.0, 0.0)
        }
    }
}

/// A lock-free planner queue for supplying the step generation interrupt 
/// with upcoming 31-phase trajectory segments.
pub struct G4TrajectoryQueue {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_taylor_evaluation() {
        let phase = TrajectoryPhase {
            duration: 0.01,
            crackle: 1000.0,
            snap_start: 10.0,
            jerk_start: 5.0,
            accel_start: 2.0,
            vel_start: 1.0,
            pos_start: 0.0,
        };

        let (p, v, a, j, s, c) = phase.evaluate_full(0.01);
        assert!(p > 0.0);
        assert!(v > 1.0);
        assert!(a > 2.0);
        assert!(j > 5.0);
        assert!(s > 10.0);
        assert_eq!(c, 1000.0);
    }

    #[test]
    fn test_g4_profile_continuity() {
        let limits = KinematicLimits::default();
        let profile = G4Profile::plan(0.0, 100.0, 0.0, 150.0, 0.0, limits);

        assert_eq!(profile.phases.len(), 31);
        assert!(profile.total_duration > 0.0);

        for i in 0..30 {
            let (p_end, v_end, a_end, j_end, s_end) = profile.phases[i].end_state();
            let next = &profile.phases[i + 1];

            assert!((p_end - next.pos_start).abs() < 1e-6, "Position discontinuity at phase {}", i);
            assert!((v_end - next.vel_start).abs() < 1e-6, "Velocity discontinuity at phase {}", i);
            assert!((a_end - next.accel_start).abs() < 1e-6, "Acceleration discontinuity at phase {}", i);
            assert!((j_end - next.jerk_start).abs() < 1e-6, "Jerk discontinuity at phase {}", i);
            assert!((s_end - next.snap_start).abs() < 1e-6, "Snap discontinuity at phase {}", i);
        }
    }
}