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

pub trait MatrixBlock<T, const R: usize, const C: usize> {
    fn set_block<const R2: usize, const C2: usize>(
        &mut self,
        row_start: usize,
        col_start: usize,
        block: &Matrix<T, R2, C2>,
    );
}

impl<T, const R: usize, const C: usize> MatrixBlock<T, R, C> for Matrix<T, R, C>
where
    T: Copy + Default,
{
    fn set_block<const R2: usize, const C2: usize>(
        &mut self,
        row_start: usize,
        col_start: usize,
        block: &Matrix<T, R2, C2>,
    ) {
        for i in 0..R2 {
            for j in 0..C2 {
                self[(row_start + i, col_start + j)] = block[(i, j)];
            }
        }
    }
}