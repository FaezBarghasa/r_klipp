#![deny(clippy::all)]
#![deny(warnings)]

//! # 16.16 Fixed-Point Number Implementation
//!
//! Provides a `Fixed16_16` type representing a 32-bit fixed-point number with
//! 16 integer bits and 16 fractional bits. This allows for efficient, deterministic
//! arithmetic on microcontrollers without a hardware FPU.
//!
//! ## Safety
//!
//! All arithmetic operations are implemented using saturating variants (`saturating_add`,
//! `saturating_sub`, etc.) to prevent panics from overflows and ensure the
//! controller continues to operate predictably even under extreme error conditions.
//!
//! ## Performance
//!
//! Multiplication and division are the most expensive operations. `mul` requires a
//! 64-bit intermediate product, and `div` requires a 64-bit dividend. These are
//! still significantly faster and more deterministic than software floating-point
//! operations on most Cortex-M MCUs.

use core::ops::{Add, Div, Mul, Sub, Neg};
use fixed::types::I16F16;

/// A 16.16 fixed-point number, wrapping `fixed::types::I16F16`.
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Fixed16_16(pub I16F16);

impl Fixed16_16 {
    /// The value zero.
    pub const ZERO: Self = Self(I16F16::ZERO);
    /// The value one.
    pub const ONE: Self = Self(I16F16::ONE);

    /// Creates a `Fixed16_16` from an `f32` float.
    pub fn from_float(f: f32) -> Self {
        Self(I16F16::from_num(f))
    }

    /// Converts the `Fixed16_16` back to an `f32` float.
    pub fn to_float(self) -> f32 {
        self.0.to_num::<f32>()
    }
}

impl Add for Fixed16_16 {
    type Output = Self;
    fn add(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_add(rhs.0))
    }
}

impl Sub for Fixed16_16 {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_sub(rhs.0))
    }
}

impl Mul for Fixed16_16 {
    type Output = Self;
    fn mul(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_mul(rhs.0))
    }
}

impl Div for Fixed16_16 {
    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        Self(self.0.saturating_div(rhs.0))
    }
}

impl Neg for Fixed16_16 {
    type Output = Self;
    fn neg(self) -> Self::Output {
        Self(self.0.saturating_neg())
    }
}
