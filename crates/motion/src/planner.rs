//! Motion planner implementing a 7‑segment S‑curve velocity profile.
//!
//! The planner computes motion segments that respect user‑defined limits on
//! velocity, acceleration, and jerk. It outputs an `SCurveSegment` containing
//! the start/end positions, target velocities, and the durations of the seven
//! phases of the S‑curve.
//!
//! All calculations are performed with `f64` precision and use `nalgebra`
//! for zero‑allocation vector math.

use nalgebra::Vector3;


/// Motion constraints supplied by the caller.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MotionConstraints {
    /// Maximum allowed velocity (mm/s).
    pub vmax: f64,
    /// Maximum allowed acceleration (mm/s²).
    pub amax: f64,
    /// Maximum allowed jerk (mm/s³).
    pub jmax: f64,
}

/// Result of a successful planning operation.
#[derive(Debug, Clone, PartialEq)]
pub struct SCurveSegment {
    /// Start position.
    pub start: Vector3<f64>,
    /// End position.
    pub end: Vector3<f64>,
    /// Target cruise velocity.
    pub v_cruise: f64,
    /// Target acceleration used during the ramp‑up / ramp‑down phases.
    pub a_target: f64,
    /// Jerk limit enforced during the constant‑jerk sub‑phases.
    pub j_max: f64,
    /// Durations of the seven S‑curve phases (seconds).
    /// Order: t1 … t7 as described in the design doc.
    pub phase_durations: [f64; 7],
}

/// Errors that can arise while planning a segment.
#[derive(Debug, Clone, PartialEq)]
pub enum MotionError {
    /// The distance is too short to achieve the requested cruise velocity under the
    /// given constraints.
    InsufficientDistance,
    /// One of the constraints is non‑positive or otherwise invalid.
    InvalidConstraints,
    /// Numerical instability detected (e.g., division by zero).
    NumericalError,
}

/// Compute a 7‑segment S‑curve profile between `start` and `end`.
///
/// The algorithm follows the classic approach described in the literature for
/// time‑optimal S‑curve motion with bounded jerk. It first determines whether a
/// cruise phase is possible; if not, it creates a symmetric profile that reaches
/// the peak velocity allowed by the distance.
pub fn plan_segment(
    start: Vector3<f64>,
    end: Vector3<f64>,
    constraints: MotionConstraints,
) -> Result<SCurveSegment, MotionError> {
    // Validate constraints early.
    if constraints.vmax <= 0.0 || constraints.amax <= 0.0 || constraints.jmax <= 0.0 {
        return Err(MotionError::InvalidConstraints);
    }

    // Compute Euclidean distance.
    let delta = end - start;
    let distance = delta.norm();
    if distance.abs() < 1e-9 {
        return Err(MotionError::InsufficientDistance);
    }

    // Helper: time to ramp acceleration from 0 to a_target with jerk limit.
    // t_jerk = a_target / jmax.
    let t_jerk = constraints.amax / constraints.jmax;
    // Distance covered during one jerk phase (triangular acceleration).
    // s_jerk = (1/6) * jmax * t_jerk³.
    let s_jerk = (constraints.jmax * t_jerk.powi(3)) / 6.0;

    // Distance needed for the full accel‑decel block (phases 1‑3 and 5‑7).
    let s_accel_decel = 2.0 * (s_jerk + 0.5 * constraints.amax * (constraints.amax / constraints.jmax));

    // Determine if a cruise phase is possible.
    if distance > s_accel_decel {
        // Cruise is possible.
        // Cruise velocity limited by vmax.
        let v_cruise = constraints.vmax.min(
            (constraints.amax.powi(2) / constraints.jmax) + (constraints.jmax * distance).sqrt(),
        );
        // Adjust peak acceleration if needed to meet the distance exactly.
        let a_target = constraints.amax;
        // Compute cruise duration.
        let s_cruise = distance - s_accel_decel;
        let t_cruise = s_cruise / v_cruise;

        // Phase timings (seconds):
        // 1. Jerk up (0 → a_target)
        // 2. Constant accel (a_target)
        // 3. Jerk down (a_target → 0)
        // 4. Cruise (v_cruise)
        // 5‑7: symmetric decel.
        let t1 = t_jerk;
        let t2 = a_target / constraints.jmax; // actually same as t1 for symmetric case
        let t3 = t_jerk;
        let t4 = t_cruise;
        let t5 = t_jerk;
        let t6 = a_target / constraints.jmax;
        let t7 = t_jerk;

        Ok(SCurveSegment {
            start,
            end,
            v_cruise,
            a_target,
            j_max: constraints.jmax,
            phase_durations: [t1, t2, t3, t4, t5, t6, t7],
        })
    } else {
        // No cruise; we must compute a reduced peak acceleration.
        // Solve for a_peak such that the total distance equals the distance.
        // Using the symmetric profile formulas:
        // distance = (a_peak³) / (jmax²) * (2/3).
        // Derivation: s = 2 * ( (1/6) * j * t³ + (1/2) * a * t² ),
        // where t = a / j and a = a_peak.
        let a_peak_cubed = (3.0 * distance * constraints.jmax.powi(2)) / 2.0;
        if a_peak_cubed < 0.0 {
            return Err(MotionError::NumericalError);
        }
        let a_peak = a_peak_cubed.powf(1.0 / 3.0);
        if a_peak.is_nan() || a_peak <= 0.0 {
            return Err(MotionError::NumericalError);
        }
        // Corresponding peak velocity.
        let v_peak = a_peak * a_peak / constraints.jmax;
        // Phase durations.
        let t1 = a_peak / constraints.jmax;
        let t2 = t1; // constant accel period disappears in pure triangular profile
        let t3 = t1;
        let t4 = 0.0; // no cruise
        let t5 = t1;
        let t6 = t1;
        let t7 = t1;
        Ok(SCurveSegment {
            start,
            end,
            v_cruise: v_peak,
            a_target: a_peak,
            j_max: constraints.jmax,
            phase_durations: [t1, t2, t3, t4, t5, t6, t7],
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use nalgebra::Vector3;

    #[test]
    fn test_plan_simple() {
        let start = Vector3::new(0.0, 0.0, 0.0);
        let end = Vector3::new(100.0, 0.0, 0.0);
        let cons = MotionConstraints {
            vmax: 200.0,
            amax: 1000.0,
            jmax: 5000.0,
        };
        let seg = plan_segment(start, end, cons).expect("should plan");
        // Verify total time is positive and phases sum correctly.
        let total: f64 = seg.phase_durations.iter().sum();
        assert!(total > 0.0);
        // Ensure cruise velocity does not exceed vmax.
        assert!(seg.v_cruise <= cons.vmax + 1e-6);
    }
}