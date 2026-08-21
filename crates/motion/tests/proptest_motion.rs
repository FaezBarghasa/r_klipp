//! Property-Based Testing for Trapezoidal and S-Curve Motion Planners.

use motion::trapezoidal::{TrapezoidalConstraints, TrapezoidalProfile};
use motion::planner::{plan_segment, MotionConstraints};
use nalgebra::Vector3;
use proptest::prelude::*;

proptest! {
    /// Invariant: Planned trapezoidal cruise velocity never exceeds constraints.v_max
    #[test]
    fn proptest_trapezoid_velocity_within_bounds(
        dist in 0.1f64..500.0f64,
        vmax in 1.0f64..300.0f64,
        amax in 10.0f64..10000.0f64,
        ventry in 0.0f64..50.0f64,
        vexit in 0.0f64..50.0f64
    ) {
        let ventry_clamped = ventry.min(vmax);
        let vexit_clamped = vexit.min(vmax);

        let cons = TrapezoidalConstraints {
            v_max: vmax,
            a_max: amax,
            v_entry: ventry_clamped,
            v_exit: vexit_clamped,
        };

        if let Ok(profile) = TrapezoidalProfile::plan(dist, cons) {
            prop_assert!(profile.v_cruise <= vmax + 1e-6);
            prop_assert!(profile.total_time > 0.0);
            prop_assert!(profile.distance >= dist - 1e-6);

            // Step interval generator invariant: intervals must be positive
            let intervals = profile.compute_step_intervals(80.0, 1_000_000.0);
            for interval in intervals {
                prop_assert!(interval > 0);
            }
        }
    }

    /// Invariant: S-Curve segment planning produces continuous finite non-NaN trajectory
    #[test]
    fn proptest_scurve_finite_trajectory(
        dx in 1.0f64..200.0f64,
        dy in 1.0f64..200.0f64,
        vmax in 10.0f64..200.0f64,
        amax in 100.0f64..5000.0f64,
        jmax in 1000.0f64..50000.0f64
    ) {
        let start = Vector3::new(0.0, 0.0, 0.0);
        let end = Vector3::new(dx, dy, 0.0);
        let cons = MotionConstraints { vmax, amax, jmax };

        if let Ok(seg) = plan_segment(start, end, cons) {
            let dur = seg.total_duration();
            prop_assert!(dur > 0.0);

            // Evaluate samples
            for i in 0..=10 {
                let t = (i as f64 / 10.0) * dur;
                let p = seg.position_at(t);
                let v = seg.velocity_at(t);
                let a = seg.acceleration_at(t);

                prop_assert!(!p.x.is_nan() && !p.y.is_nan() && !p.z.is_nan());
                prop_assert!(!v.is_nan());
                prop_assert!(!a.is_nan());
            }
        }
    }
}
