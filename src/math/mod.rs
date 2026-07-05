// Copyright 2024 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     https://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

pub use nalgebra::{
    Matrix3, Matrix4, Matrix6, Point3, SMatrix, SVector, Vector3, Vector6, U1, U3, U6,
};

pub mod goertzel;
pub mod matrix_utils;
pub mod svd;

pub type Matrix<const R: usize, const C: usize> = SMatrix<f64, R, C>;
pub type Vector<const D: usize> = SVector<f64, D>;
pub type Point = Point3<f64>;

impl Vector3<f64> {
    pub fn to_skew_symmetric(&self) -> Matrix3<f64> {
        Matrix3::new(0.0, -self.z, self.y, self.z, 0.0, -self.x, -self.y, self.x, 0.0)
    }
}