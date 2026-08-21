//! Classical Trapezoidal Motion Planner (G1/G2) with exact discrete step tick generation.

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TrapezoidalConstraints {
    pub v_max: f64,
    pub a_max: f64,
    pub v_entry: f64,
    pub v_exit: f64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TrapezoidalProfile {
    pub distance: f64,
    pub v_entry: f64,
    pub v_cruise: f64,
    pub v_exit: f64,
    pub a_max: f64,
    pub t_accel: f64,
    pub d_accel: f64,
    pub t_cruise: f64,
    pub d_cruise: f64,
    pub t_decel: f64,
    pub d_decel: f64,
    pub total_time: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrapezoidalError {
    InvalidConstraints,
    ZeroDistance,
}

impl TrapezoidalProfile {
    /// Plans a constant acceleration/deceleration trapezoidal move.
    pub fn plan(distance: f64, mut constraints: TrapezoidalConstraints) -> Result<Self, TrapezoidalError> {
        if distance <= 0.0 {
            return Err(TrapezoidalError::ZeroDistance);
        }
        if constraints.v_max <= 0.0 || constraints.a_max <= 0.0 {
            return Err(TrapezoidalError::InvalidConstraints);
        }

        // Clamp entry and exit velocities to v_max
        constraints.v_entry = constraints.v_entry.clamp(0.0, constraints.v_max);
        constraints.v_exit = constraints.v_exit.clamp(0.0, constraints.v_max);

        let a = constraints.a_max;
        let v_entry = constraints.v_entry;
        let v_exit = constraints.v_exit;
        let v_max = constraints.v_max;

        // Minimum distance required to accelerate from v_entry to v_max and decelerate to v_exit:
        // d_accel = (v_max^2 - v_entry^2) / (2 * a)
        // d_decel = (v_max^2 - v_exit^2) / (2 * a)
        let d_accel_full = (v_max * v_max - v_entry * v_entry) / (2.0 * a);
        let d_decel_full = (v_max * v_max - v_exit * v_exit) / (2.0 * a);

        let (v_cruise, d_accel, d_decel, d_cruise) = if d_accel_full + d_decel_full <= distance {
            // Full trapezoid with cruise phase
            let d_cruise = distance - (d_accel_full + d_decel_full);
            (v_max, d_accel_full, d_decel_full, d_cruise)
        } else {
            // Triangular profile (no cruise phase, peak velocity reached is lower than v_max)
            // distance = (v_peak^2 - v_entry^2)/(2a) + (v_peak^2 - v_exit^2)/(2a)
            // 2a * distance = 2 * v_peak^2 - v_entry^2 - v_exit^2
            // v_peak^2 = (2a * distance + v_entry^2 + v_exit^2) / 2
            let v_peak_sq = (2.0 * a * distance + v_entry * v_entry + v_exit * v_exit) / 2.0;
            let v_peak = if v_peak_sq > 0.0 { v_peak_sq.sqrt().min(v_max) } else { v_entry };
            let d_accel = (v_peak * v_peak - v_entry * v_entry).max(0.0) / (2.0 * a);
            let d_decel = (v_peak * v_peak - v_exit * v_exit).max(0.0) / (2.0 * a);
            (v_peak, d_accel, d_decel, 0.0)
        };

        let t_accel = if a > 0.0 { (v_cruise - v_entry).abs() / a } else { 0.0 };
        let t_decel = if a > 0.0 { (v_cruise - v_exit).abs() / a } else { 0.0 };
        let t_cruise = if v_cruise > 0.0 { d_cruise / v_cruise } else { 0.0 };
        let total_time = t_accel + t_cruise + t_decel;

        Ok(Self {
            distance,
            v_entry,
            v_cruise,
            v_exit,
            a_max: a,
            t_accel,
            d_accel,
            t_cruise,
            d_cruise,
            t_decel,
            d_decel,
            total_time,
        })
    }

    /// Evaluates toolhead position at time `t` (0 <= t <= total_time).
    pub fn position_at(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, self.total_time);

        if t <= self.t_accel {
            // Accel phase: s(t) = v_entry * t + 0.5 * a * t^2
            self.v_entry * t + 0.5 * self.a_max * t * t
        } else if t <= self.t_accel + self.t_cruise {
            // Cruise phase: s(t) = d_accel + v_cruise * (t - t_accel)
            let dt = t - self.t_accel;
            self.d_accel + self.v_cruise * dt
        } else {
            // Decel phase: s(t) = d_accel + d_cruise + v_cruise * dt - 0.5 * a * dt^2
            let dt = t - (self.t_accel + self.t_cruise);
            self.d_accel + self.d_cruise + self.v_cruise * dt - 0.5 * self.a_max * dt * dt
        }
    }

    /// Evaluates toolhead velocity at time `t`.
    pub fn velocity_at(&self, t: f64) -> f64 {
        let t = t.clamp(0.0, self.total_time);

        if t <= self.t_accel {
            self.v_entry + self.a_max * t
        } else if t <= self.t_accel + self.t_cruise {
            self.v_cruise
        } else {
            let dt = t - (self.t_accel + self.t_cruise);
            (self.v_cruise - self.a_max * dt).max(self.v_exit)
        }
    }

    /// Evaluates toolhead acceleration at time `t`.
    pub fn acceleration_at(&self, t: f64) -> f64 {
        if t < 0.0 || t > self.total_time {
            0.0
        } else if t <= self.t_accel {
            self.a_max
        } else if t <= self.t_accel + self.t_cruise {
            0.0
        } else {
            -self.a_max
        }
    }

    /// Computes the exact series of timer ticks (ARR values) for each step pulse.
    pub fn compute_step_intervals(&self, steps_per_mm: f64, timer_freq_hz: f64) -> heapless::Vec<u32, 512> {
        let mut intervals = heapless::Vec::new();
        let total_steps = (self.distance * steps_per_mm).round() as usize;

        if total_steps == 0 || self.total_time <= 0.0 {
            return intervals;
        }

        let mut last_t = 0.0;
        let mut current_pos = 0.0;

        for step in 1..=total_steps.min(512) {
            current_pos = step as f64 / steps_per_mm;

            // Invert position function numerically with binary search
            let mut low = last_t;
            let mut high = self.total_time;
            for _ in 0..16 {
                let mid = (low + high) * 0.5;
                if self.position_at(mid) < current_pos {
                    low = mid;
                } else {
                    high = mid;
                }
            }

            let t_step = (low + high) * 0.5;
            let delta_t = (t_step - last_t).max(1.0 / timer_freq_hz);
            let ticks = (delta_t * timer_freq_hz).round() as u32;

            let _ = intervals.push(ticks.max(1));
            last_t = t_step;
        }

        intervals
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trapezoidal_planner_full_cruise() {
        let cons = TrapezoidalConstraints {
            v_max: 100.0,
            a_max: 1000.0,
            v_entry: 0.0,
            v_exit: 0.0,
        };

        let profile = TrapezoidalProfile::plan(50.0, cons).expect("planning failed");
        assert_eq!(profile.v_cruise, 100.0);
        assert!(profile.t_cruise > 0.0);
        assert!((profile.position_at(profile.total_time) - 50.0).abs() < 1e-4);
        assert!((profile.velocity_at(profile.total_time) - 0.0).abs() < 1e-4);
    }

    #[test]
    fn test_trapezoidal_triangular_short_move() {
        let cons = TrapezoidalConstraints {
            v_max: 200.0,
            a_max: 1000.0,
            v_entry: 0.0,
            v_exit: 0.0,
        };

        // Short move of 2mm cannot reach 200mm/s with 1000mm/s^2 accel
        let profile = TrapezoidalProfile::plan(2.0, cons).expect("planning failed");
        assert!(profile.v_cruise < 200.0);
        assert_eq!(profile.t_cruise, 0.0);
        assert!((profile.position_at(profile.total_time) - 2.0).abs() < 1e-4);
    }

    #[test]
    fn test_step_interval_generation() {
        let cons = TrapezoidalConstraints {
            v_max: 50.0,
            a_max: 500.0,
            v_entry: 0.0,
            v_exit: 0.0,
        };

        let profile = TrapezoidalProfile::plan(1.0, cons).unwrap();
        let intervals = profile.compute_step_intervals(80.0, 1_000_000.0); // 80 steps/mm, 1MHz timer
        assert_eq!(intervals.len(), 80);
        for tick in &intervals {
            assert!(*tick > 0);
        }
    }
}
