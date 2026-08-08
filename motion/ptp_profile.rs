
use crate::kinematics::robotics::articulated::ArticulatedKinematics;
use crate::kinematics::robotics::scara::ScaraKinematics;
use crate::kinematics::robotics::delta::DeltaKinematics;
use crate::kinematics::Kinematics;
use crate::utils::s_curve::SCurve;
use crate::utils::trapezoidal::Trapezoidal;
use num_traits::Float;

pub trait PtpProfile<K: Kinematics> {
    fn new(kinematics: K) -> Self;
    fn plan_motion(&self, start: &[f32], end: &[f32]) -> Result<Vec<f32>, &'static str>;
}

pub struct SCurvePtp<K: Kinematics> {
    kinematics: K,
    s_curve: SCurve,
}

impl<K: Kinematics> PtpProfile<K> for SCurvePtp<K> {
    fn new(kinematics: K) -> Self {
        Self {
            kinematics,
            s_curve: SCurve::new(),
        }
    }

    fn plan_motion(&self, start: &[f32], end: &[f32]) -> Result<Vec<f32>, &'static str> {
        let mut motion_plan = Vec::new();
        let num_axes = start.len();

        for i in 0..num_axes {
            let distance = end[i] - start[i];
            let motion = self.s_curve.plan(distance, 1.0, 1.0, 1.0);
            motion_plan.extend(motion);
        }

        Ok(motion_plan)
    }
}

pub struct TrapezoidalPtp<K: Kinematics> {
    kinematics: K,
    trapezoidal: Trapezoidal,
}

impl<K: Kinematics> PtpProfile<K> for TrapezoidalPtp<K> {
    fn new(kinematics: K) -> Self {
        Self {
            kinematics,
            trapezoidal: Trapezoidal::new(),
        }
    }

    fn plan_motion(&self, start: &[f32], end: &[f32]) -> Result<Vec<f32>, &'static str> {
        let mut motion_plan = Vec::new();
        let num_axes = start.len();

        for i in 0..num_axes {
            let distance = end[i] - start[i];
            let motion = self.trapezoidal.plan(distance, 1.0, 1.0);
            motion_plan.extend(motion);
        }

        Ok(motion_plan)
    }
}
