#![no_std]
#![feature(const_generics)]

pub mod five_axis;
pub mod wire_edm;
pub mod ph_blending;
pub mod matrix;
pub mod robotics;

pub enum KinematicsError {
    Singularity,
    Unreachable,
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
