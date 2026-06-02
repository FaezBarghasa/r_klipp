// crates/motion/src/g4_planner.rs

#[derive(Copy, Clone, Debug, Default)]
pub struct TrajectoryPhase {
    pub duration: f64,
    pub crackle: f64,
    pub snap_start: f64,
    pub jerk_start: f64,
    pub accel_start: f64,
    pub vel_start: f64,
    pub pos_start: f64,
}

pub struct G4Profile {
    pub phases: heapless::Vec<TrajectoryPhase, 32>,
    pub total_duration: f64,
}

impl G4Profile {
    /// Builds a multi-phase velocity profiles boundary system to transition between states.
    pub fn generate_profile(
        start_vel: f64, 
        _end_vel: f64, 
        max_vel: f64, 
        max_accel: f64, 
        max_jerk: f64, 
        max_snap: f64, 
        max_crackle: f64
    ) -> Self {
        let c_limit = max_crackle.max(1.0);
        let s_limit = max_snap.max(1.0);
        let j_limit = max_jerk.max(1.0);
        let a_limit = max_accel.max(1.0);

        // Calculate segment durations for the high-order profile
        let t_c = (s_limit / c_limit).min(0.05);
        let t_s = ((j_limit / s_limit) - t_c).max(0.0).min(0.05);
        let t_j = ((a_limit / j_limit) - 2.0 * t_c - t_s).max(0.0).min(0.1);
        
        let ramp_time = 4.0 * t_c + 2.0 * t_s + t_j;
        let v_diff = max_vel - start_vel;
        let t_a = (v_diff / a_limit - ramp_time).max(0.0).min(0.2);

        let mut phase_definitions = [(0.0, 0.0); 31];
        
        // 15 acceleration ramp phases
        phase_definitions[0] = (t_c, c_limit);
        phase_definitions[1] = (t_s, 0.0);
        phase_definitions[2] = (t_c, -c_limit);
        phase_definitions[3] = (t_j, 0.0);
        phase_definitions[4] = (t_c, -c_limit);
        phase_definitions[5] = (t_s, 0.0);
        phase_definitions[6] = (t_c, c_limit);
        phase_definitions[7] = (t_a, 0.0);
        phase_definitions[8] = (t_c, c_limit);
        phase_definitions[9] = (t_s, 0.0);
        phase_definitions[10] = (t_c, -c_limit);
        phase_definitions[11] = (t_j, 0.0);
        phase_definitions[12] = (t_c, -c_limit);
        phase_definitions[13] = (t_s, 0.0);
        phase_definitions[14] = (t_c, c_limit);
        
        // 1 cruise phase
        phase_definitions[15] = (0.5, 0.0);
        
        // 15 deceleration ramp phases (mirroring acceleration, with signs inverted)
        phase_definitions[16] = (t_c, -c_limit);
        phase_definitions[17] = (t_s, 0.0);
        phase_definitions[18] = (t_c, c_limit);
        phase_definitions[19] = (t_j, 0.0);
        phase_definitions[20] = (t_c, c_limit);
        phase_definitions[21] = (t_s, 0.0);
        phase_definitions[22] = (t_c, -c_limit);
        phase_definitions[23] = (t_a, 0.0);
        phase_definitions[24] = (t_c, -c_limit);
        phase_definitions[25] = (t_s, 0.0);
        phase_definitions[26] = (t_c, c_limit);
        phase_definitions[27] = (t_j, 0.0);
        phase_definitions[28] = (t_c, c_limit);
        phase_definitions[29] = (t_s, 0.0);
        phase_definitions[30] = (t_c, -c_limit);

        let mut current_pos = 0.0;
        let mut current_vel = start_vel;
        let mut current_accel = 0.0;
        let mut current_jerk = 0.0;
        let mut current_snap = 0.0;
        let mut total_duration = 0.0;

        let mut phases = heapless::Vec::new();

        for i in 0..31 {
            let (dur, crackle) = phase_definitions[i];
            
            let phase = TrajectoryPhase {
                duration: dur,
                crackle,
                snap_start: current_snap,
                jerk_start: current_jerk,
                accel_start: current_accel,
                vel_start: current_vel,
                pos_start: current_pos,
            };
            let _ = phases.push(phase);

            let t = dur;
            let t2 = t * t;
            let t3 = t2 * t;
            let t4 = t3 * t;
            let t5 = t4 * t;

            let next_snap = current_snap + crackle * t;
            let next_jerk = current_jerk + current_snap * t + 0.5 * crackle * t2;
            let next_accel = current_accel + current_jerk * t + 0.5 * current_snap * t2 + (1.0 / 6.0) * crackle * t3;
            let next_vel = current_vel + current_accel * t + 0.5 * current_jerk * t2 + (1.0 / 6.0) * current_snap * t3 + (1.0 / 24.0) * crackle * t4;
            let next_pos = current_pos + current_vel * t + 0.5 * current_accel * t2 + (1.0 / 6.0) * current_jerk * t3 + (1.0 / 24.0) * current_snap * t4 + (1.0 / 120.0) * crackle * t5;

            current_snap = next_snap;
            current_jerk = next_jerk;
            current_accel = next_accel;
            current_vel = next_vel;
            current_pos = next_pos;
            total_duration += dur;
        }

        Self {
            phases,
            total_duration,
        }
    }

    /// Evaluates position, velocity, and acceleration values at a given time index delta_t.
    pub fn evaluate_state(&self, mut t: f64) -> (f64, f64, f64) {
        for phase in &self.phases {
            if t <= phase.duration {
                let dt2 = t * t;
                let dt3 = dt2 * t;
                let dt4 = dt3 * t;
                let dt5 = dt4 * t;
                
                let p = phase.pos_start + phase.vel_start * t + 0.5 * phase.accel_start * dt2 
                    + (1.0 / 6.0) * phase.jerk_start * dt3 + (1.0 / 24.0) * phase.snap_start * dt4 
                    + (1.0 / 120.0) * phase.crackle * dt5;
                    
                let v = phase.vel_start + phase.accel_start * t + 0.5 * phase.jerk_start * dt2 
                    + (1.0 / 6.0) * phase.snap_start * dt3 + (1.0 / 24.0) * phase.crackle * dt4;
                    
                let a = phase.accel_start + phase.jerk_start * t + 0.5 * phase.snap_start * dt2 
                    + (1.0 / 6.0) * phase.crackle * dt3;
                return (p, v, a);
            }
            t -= phase.duration;
        }
        
        // If out of bounds, return the terminal state values of the final phase
        if let Some(final_phase) = self.phases.last() {
            let dur = final_phase.duration;
            let p = final_phase.pos_start + final_phase.vel_start * dur;
            return (p, final_phase.vel_start, 0.0);
        }
        (0.0, 0.0, 0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_g4_profile_generation() {
        let profile = G4Profile::generate_profile(0.0, 0.0, 100.0, 2000.0, 20000.0, 50000.0, 100000.0);
        
        // Enforce 31-phase array size constraint
        assert_eq!(profile.phases.len(), 31);
        
        // Check start, mid, end evaluation
        let (p_start, v_start, a_start) = profile.evaluate_state(0.0);
        assert_eq!(p_start, 0.0);
        assert_eq!(v_start, 0.0);
        assert_eq!(a_start, 0.0);

        let (p_end, v_end, _) = profile.evaluate_state(profile.total_duration);
        assert!(p_end > 0.0);
        assert!(v_end >= 0.0);
    }
}
