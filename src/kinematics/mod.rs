#![feature(const_generics)]

#[derive(Debug)]
pub enum KinematicsError {
    Unreachable,
    Singularity,
}

pub trait Kinematics<const N_JOINTS: usize, const N_CARTESIAN: usize> {
    fn forward(&self, joints: &[f32; N_JOINTS]) -> Result<[f32; N_CARTESIAN], KinematicsError>;
    fn inverse(&self, cartesian: &[f32; N_CARTESIAN]) -> Result<[f32; N_JOINTS], KinematicsError>;
}

pub struct CartesianKinematics;

impl Kinematics<3, 3> for CartesianKinematics {
    fn forward(&self, joints: &[f32; 3]) -> Result<[f32; 3], KinematicsError> {
        Ok(*joints)
    }

    fn inverse(&self, cartesian: &[f32; 3]) -> Result<[f32; 3], KinematicsError> {
        Ok(*cartesian)
    }
}
