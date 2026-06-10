use heapless::Vec;

#[derive(Debug, Clone, Copy)]
pub struct TrajectoryPhase {
    pub duration: f64,
    pub crackle: f64,
    pub snap_start: f64,
    pub jerk_start: f64,
    pub accel_start: f64,
    pub vel_start: f64,
    pub pos_start: f64,
}

impl TrajectoryPhase {
    pub fn evaluate_state(&self, t: f64) -> (f64, f64, f64) {
        let t2 = t * t;
        let t3 = t2 * t;
        let t4 = t3 * t;
        let t5 = t4 * t;

        let pos = self.pos_start
            + self.vel_start * t
            + 0.5 * self.accel_start * t2
            + (1.0 / 6.0) * self.jerk_start * t3
            + (1.0 / 24.0) * self.snap_start * t4
            + (1.0 / 120.0) * self.crackle * t5;

        let vel = self.vel_start
            + self.accel_start * t
            + 0.5 * self.jerk_start * t2
            + (1.0 / 6.0) * self.snap_start * t3
            + (1.0 / 24.0) * self.crackle * t4;

        let accel = self.accel_start
            + self.jerk_start * t
            + 0.5 * self.snap_start * t2
            + (1.0 / 6.0) * self.crackle * t3;

        (pos, vel, accel)
    }
}

pub struct G4Profile {
    pub phases: Vec<TrajectoryPhase, 32>,
    pub total_duration: f64,
}

impl G4Profile {
    pub fn generate_profile(
        start_vel: f64,
        end_vel: f64,
        _max_vel: f64,
        _max_accel: f64,
        _max_jerk: f64,
        max_snap: f64,
        max_crackle: f64,
    ) -> Self {
        let mut phases: Vec<TrajectoryPhase, 32> = Vec::new();
        let mut total_duration = 0.0;
        
        let mut current_pos = 0.0;
        let mut current_vel = start_vel;
        let mut current_accel = 0.0;
        let mut current_jerk = 0.0;
        let mut current_snap = 0.0;
        
        let step_time = 0.05; 
        
        for _ in 0..31 {
            if (end_vel - current_vel).abs() < 1e-6 {
                break;
            }

            let crackle = if current_vel < end_vel { max_crackle } else { -max_crackle };
            
            let phase = TrajectoryPhase {
                duration: step_time,
                crackle,
                snap_start: current_snap,
                jerk_start: current_jerk,
                accel_start: current_accel,
                vel_start: current_vel,
                pos_start: current_pos,
            };
            
            let (next_pos, next_vel, next_accel) = phase.evaluate_state(step_time);
            
            let unbounded_snap = current_snap + crackle * step_time;
            current_snap = unbounded_snap.clamp(-max_snap, max_snap);
            current_jerk += current_snap * step_time + 0.5 * crackle * step_time * step_time;
            current_accel = next_accel;
            current_vel = next_vel;
            current_pos = next_pos;
            
            if phases.push(phase).is_err() {
                break;
            }
            total_duration += step_time;
        }

        Self {
            phases,
            total_duration,
        }
    }
}