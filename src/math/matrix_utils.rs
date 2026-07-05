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

use crate::math::Matrix;
use nalgebra::Scalar;

pub trait MatrixBlock<T, const R: usize, const C: usize>
where
    T: Scalar,
{
    fn set_block<const R2: usize, const C2: usize>(
        &mut self,
        row_start: usize,
        col_start: usize,
        block: &Matrix<R2, C2>,
    );
}

impl<T, const R: usize, const C: usize> MatrixBlock<T, R, C> for Matrix<R, C>
where
    T: Scalar + Copy + Default,
{
    fn set_block<const R2: usize, const C2: usize>(
        &mut self,
        row_start: usize,
        col_start: usize,
        block: &Matrix<R2, C2>,
    ) {
        self.view_mut((row_start, col_start), (R2, C2))
            .copy_from(block);
    }
}