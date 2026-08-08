//! Product of Exponentials (PoE) Forward Kinematics.

use crate::motion::kinematics::math::se3::{Transform, Twist, exponential_map};

pub struct PoeKinematics<const N: usize> {
    screws: [Twist; N],
    m: Transform,
}

impl<const N: usize> PoeKinematics<N> {
    pub fn new(screws: [Twist; N], m: Transform) -> Self {
        Self { screws, m }
    }

    pub fn fk(&self, thetas: &[f32; N]) -> Transform {
        let mut t = Transform::identity();
        for i in 0..N {
            t = t * exponential_map(&self.screws[i], thetas[i]);
        }
        t * self.m
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::motion::kinematics::math::se3::{Twist, Transform};

    #[test]
    fn test_poe_fk_6_axis() {
        // UR5 parameters
        let screws = [
            Twist { v: [0.0, 0.0, 0.0], w: [0.0, 0.0, 1.0] },
            Twist { v: [0.0, 0.0, 0.0], w: [0.0, 1.0, 0.0] },
            Twist { v: [0.0, 0.0, 0.0], w: [0.0, 1.0, 0.0] },
            Twist { v: [0.0, 0.0, 0.0], w: [0.0, 1.0, 0.0] },
            Twist { v: [0.0, 0.0, 0.0], w: [0.0, 0.0, 1.0] },
            Twist { v: [0.0, 0.0, 0.0], w: [0.0, 1.0, 0.0] },
        ];
        let m = Transform {
            rot: [
                [-1.0, 0.0, 0.0],
                [0.0, 0.0, 1.0],
                [0.0, 1.0, 0.0],
            ],
            trans: [0.817, 0.191, -0.006],
        };

        let kinematics = PoeKinematics::new(screws, m);
        let thetas = [0.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let fk = kinematics.fk(&thetas);

        assert!((fk.trans[0] - 0.817).abs() < 1e-3);
        assert!((fk.trans[1] - 0.191).abs() < 1e-3);
        assert!((fk.trans[2] - -0.006).abs() < 1e-3);
    }
}
