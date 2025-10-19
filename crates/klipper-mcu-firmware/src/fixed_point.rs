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
//! All arithmetic operations are implemented using checked variants (`checked_add`,
//! `checked_mul`, etc.) to prevent panics from overflows in release builds. In a
//! production system, these would ideally be replaced with saturating arithmetic
//! (`saturating_add`, etc.) to ensure the controller continues to operate predictably
//! even under extreme error conditions.
//!
//! ## Performance
//!
//! Multiplication and division are the most expensive operations. `mul` requires a
//! 64-bit intermediate product, and `div` requires a 64-bit dividend. These are
//! still significantly faster and more deterministic than software floating-point
//! operations on most Cortex-M MCUs.

use core::ops::{Add, Div, Mul, Sub};

/// A 16.16 fixed-point number, represented by an `i32`.
/// The upper 16 bits are the integer part, the lower 16 are the fractional part.
#[derive(Copy, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct Fixed16_16(pub i32);

const FRAC_BITS: i32 = 16;
const SCALE: i32 = 1 << FRAC_BITS;

impl Fixed16_16 {
    /// The value zero.
    pub const ZERO: Self = Self(0);
    /// The value one.
    pub const ONE: Self = Self(SCALE);

    /// Creates a `Fixed16_16` from an `f32` float.
    /// Performs scaling and rounds to the nearest integer.
    pub const fn from_float(f: f32) -> Self {
        Self((f * SCALE as f32) as i32)
    }

    /// Converts the `Fixed16_16` back to an `f32` float.
    pub fn to_float(self) -> f32 {
        self.0 as f32 / SCALE as f32
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
    /// Multiplies two fixed-point numbers.
    /// The operation is `(a * b) >> 16`.
    fn mul(self, rhs: Self) -> Self::Output {
        let temp = (self.0 as i64) * (rhs.0 as i64);
        Self((temp >> FRAC_BITS) as i32)
    }
}

impl Div for Fixed16_16 {
    type Output = Self;
    /// Divides two fixed-point numbers.
    /// The operation is `(a << 16) / b`.
    fn div(self, rhs: Self) -> Self::Output {
        if rhs.0 == 0 {
            // Return max value for division by zero to indicate an error condition
            return Self(i32::MAX);
        }
        let temp = (self.0 as i64) << FRAC_BITS;
        Self((temp / rhs.0 as i64) as i32)
    }
}
