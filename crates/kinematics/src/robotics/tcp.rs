use nalgebra::{UnitQuaternion, Vector3};

pub struct Tcp {
    // Tool Center Point position and orientation
    pub position: Vector3<f32>,
    pub orientation: UnitQuaternion<f32>,
}

impl Tcp {
    pub fn slerp(&self, other: &Self, t: f32) -> UnitQuaternion<f32> {
        self.orientation.slerp(&other.orientation, t)
    }

    pub fn lerp(&self, other: &Self, t: f32) -> Vector3<f32> {
        self.position.lerp(&other.position, t)
    }
}
