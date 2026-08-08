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

#![allow(non_snake_case)]

use crate::math::{Matrix, Vector};

const MAX_SVD_ITERATIONS: usize = 30;
const SVD_TOLERANCE: f64 = 1e-12;

pub struct SVD<const M: usize, const N: usize> {
    pub U: Matrix<M, M>,
    pub S: Vector<N>,
    pub V: Matrix<N, N>,
}

impl<const M: usize, const N: usize> SVD<M, N> {
    pub fn new(A: &Matrix<M, N>) -> Self {
        let mut U = Matrix::<M, N>::from(*A);
        let mut S = Vector::<N>::zero();
        let mut V = Matrix::<N, N>::zero();

        let mut e = Vector::<N>::zero();
        let mut g = 0.0;
        let mut scale = 0.0;
        let mut anorm = 0.0;

        for i in 0..N {
            e[i] = g;
            scale = 0.0;
            g = 0.0;
            if i < M {
                for k in i..M {
                    scale += libm::fabs(U[(k, i)]);
                }
                if scale != 0.0 {
                    let mut s = 0.0;
                    for k in i..M {
                        U[(k, i)] /= scale;
                        s += U[(k, i)] * U[(k, i)];
                    }
                    let f = U[(i, i)];
                    g = -libm::copysign(libm::sqrt(s), f);
                    let h = f * g - s;
                    U[(i, i)] = f - g;
                    for j in (i + 1)..N {
                        let mut s = 0.0;
                        for k in i..M {
                            s += U[(k, i)] * U[(k, j)];
                        }
                        let f = s / h;
                        for k in i..M {
                            U[(k, j)] += f * U[(k, i)];
                        }
                    }
                    for k in i..M {
                        U[(k, i)] *= scale;
                    }
                }
            }
            S[i] = g * scale;
            g = 0.0;
            scale = 0.0;
            if i < M && i != N - 1 {
                for k in (i + 1)..N {
                    scale += libm::fabs(U[(i, k)]);
                }
                if scale != 0.0 {
                    let mut s = 0.0;
                    for k in (i + 1)..N {
                        U[(i, k)] /= scale;
                        s += U[(i, k)] * U[(i, k)];
                    }
                    let f = U[(i, i + 1)];
                    g = -libm::copysign(libm::sqrt(s), f);
                    let h = f * g - s;
                    U[(i, i + 1)] = f - g;
                    for k in (i + 1)..N {
                        e[k] = U[(i, k)] / h;
                    }
                    for j in (i + 1)..M {
                        let mut s = 0.0;
                        for k in (i + 1)..N {
                            s += U[(j, k)] * U[(i, k)];
                        }
                        for k in (i + 1)..N {
                            U[(j, k)] += s * e[k];
                        }
                    }
                    for k in (i + 1)..N {
                        U[(i, k)] *= scale;
                    }
                }
            }
            anorm = libm::fmax(anorm, libm::fabs(S[i]) + libm::fabs(e[i]));
        }

        for i in (0..N).rev() {
            if i < N - 1 {
                if g != 0.0 {
                    for j in (i + 1)..N {
                        V[(j, i)] = (U[(i, j)] / U[(i, i + 1)]) / g;
                    }
                    for j in (i + 1)..N {
                        let mut s = 0.0;
                        for k in (i + 1)..N {
                            s += U[(i, k)] * V[(k, j)];
                        }
                        for k in (i + 1)..N {
                            V[(k, j)] += s * V[(k, i)];
                        }
                    }
                }
                for j in (i + 1)..N {
                    V[(i, j)] = 0.0;
                    V[(j, i)] = 0.0;
                }
            }
            V[(i, i)] = 1.0;
            g = e[i];
        }

        let mut U_mat = Matrix::<M, M>::identity();
        for i in (0..N).rev() {
            let l = i + 1;
            g = S[i];
            if i < N - 1 {
                for j in l..N {
                    U[(i, j)] = 0.0;
                }
            }
            if g != 0.0 {
                let mut h = U[(i, i)] * g;
                if h != 0.0 {
                    for j in l..N {
                        let mut s = 0.0;
                        for k in l..M {
                            s += U[(k, i)] * U[(k, j)];
                        }
                        let f = s / h;
                        for k in i..M {
                            U[(k, j)] += f * U[(k, i)];
                        }
                    }
                }
                for j in i..M {
                    U[(j, i)] /= g;
                }
            } else {
                for j in i..M {
                    U[(j, i)] = 0.0;
                }
            }
            U[(i, i)] += 1.0;
        }

        for k in (0..N).rev() {
            for _its in 0..MAX_SVD_ITERATIONS {
                let mut flag = true;
                let mut l = k;
                while l > 0 {
                    if libm::fabs(e[l - 1]) <= SVD_TOLERANCE * anorm {
                        flag = false;
                        break;
                    }
                    if libm::fabs(S[l - 1]) <= SVD_TOLERANCE * anorm {
                        let mut c = 0.0;
                        let mut s = 1.0;
                        for i in l..=k {
                            let f = s * e[i - 1];
                            e[i - 1] = c * e[i - 1];
                            if libm::fabs(f) <= SVD_TOLERANCE * anorm {
                                break;
                            }
                            g = S[i];
                            let h = libm::sqrt(f * f + g * g);
                            S[i] = h;
                            c = g / h;
                            s = -f / h;
                            for j in 0..M {
                                let y = U_mat[(j, l - 1)];
                                let z = U_mat[(j, i)];
                                U_mat[(j, l - 1)] = y * c + z * s;
                                U_mat[(j, i)] = -y * s + z * c;
                            }
                        }
                        break;
                    }
                    l -= 1;
                }

                if flag {
                    let mut z = S[k];
                    if l == k {
                        if z < 0.0 {
                            S[k] = -z;
                            for j in 0..N {
                                V[(j, k)] = -V[(j, k)];
                            }
                        }
                        break;
                    }

                    let x = S[l];
                    let y = S[k - 1];
                    g = e[k - 1];
                    let h = e[k];
                    let mut f = ((y - z) * (y + z) + (g - h) * (g + h)) / (2.0 * h * y);
                    g = libm::sqrt(f * f + 1.0);
                    f = ((x - z) * (x + z) + h * (y / (f + libm::copysign(g, f)) - h)) / x;

                    let mut c = 1.0;
                    let mut s = 1.0;
                    for j in l..k {
                        let i = j + 1;
                        g = e[i];
                        y = S[i];
                        h = s * g;
                        g = c * g;
                        z = libm::sqrt(f * f + h * h);
                        e[j] = z;
                        c = f / z;
                        s = h / z;
                        f = x * c + g * s;
                        g = -x * s + g * c;
                        h = y * s;
                        y *= c;
                        for jj in 0..N {
                            x = V[(jj, j)];
                            z = V[(jj, i)];
                            V[(jj, j)] = x * c + z * s;
                            V[(jj, i)] = -x * s + z * c;
                        }
                        z = libm::sqrt(f * f + h * h);
                        S[j] = z;
                        if z != 0.0 {
                            c = f / z;
                            s = h / z;
                        }
                        f = c * g + s * y;
                        x = -s * g + c * y;
                        for jj in 0..M {
                            y = U_mat[(jj, j)];
                            z = U_mat[(jj, i)];
                            U_mat[(jj, j)] = y * c + z * s;
                            U_mat[(jj, i)] = -y * s + z * c;
                        }
                    }
                    e[l] = 0.0;
                    e[k] = f;
                    S[k] = x;
                }
            }
        }

        Self { U: U_mat, S, V }
    }

    pub fn singular_values(&self) -> &Vector<N> {
        &self.S
    }

    pub fn pseudo_inverse(&self, tolerance: f64) -> Matrix<N, M> {
        let mut S_inv = Matrix::<N, M>::zero();
        for i in 0..N {
            if self.S[i] > tolerance {
                S_inv[(i, i)] = 1.0 / self.S[i];
            }
        }
        self.V * S_inv * self.U.transpose()
    }
}
