//! The [`Sample`] trait: the universal element type of a signal stream.
//!
//! See `docs/architecture-planning.md` decisions 4, 5, and 6 for the design
//! rationale. In short:
//!
//! - The trait is **deliberately smaller than `num_traits::Float`** so that
//!   integer (fixed-point) sample types can satisfy it without taking on
//!   meaningless obligations (NaN, infinity).
//! - The trait is **deliberately named `Sample` rather than `Real`** to avoid
//!   colliding with `num_traits::real::Real`, which means something different.
//! - `Sample` is satisfied by `f32`, `f64`, `i16`, `i8`, and the
//!   [`num_complex::Complex<T>`] wrapping of each. `Complex<u8>` (the
//!   RTL-SDR boundary type) is **not** a `Sample` by design — see decision 6.
//!
//! Kernels are written generically over `T: Sample` and monomorphised by the
//! compiler per concrete type. No runtime dispatch.

use core::fmt::Debug;
use core::ops::{Add, Mul, Neg, Sub};

use num_complex::Complex;

/// The universal element type of a signal stream.
///
/// `Sample` is the trait bound under which honeyeater kernels are generic. It
/// captures the small intersection of arithmetic operations that every honest
/// sample type — real or complex, floating-point or signed-integer
/// fixed-point — can support without semantic compromise.
///
/// # Implementors at 0.0.1
///
/// - Real-valued: `f32`, `f64`, `i16`, `i8`.
/// - Complex-valued: `Complex<f32>`, `Complex<f64>`, `Complex<i16>`,
///   `Complex<i8>` (impl'd per concrete type rather than via a blanket impl;
///   see the impl comment below for why).
///
/// # Not implementors
///
/// - `Complex<u8>` is **not** a `Sample`. The RTL-SDR delivers unsigned-8
///   samples biased around `127.5`, where unsigned-wrapping arithmetic gives
///   the wrong answer for signal addition. The boundary helpers debias
///   `Complex<u8>` to `Complex<i8>` or `Complex<f32>` before any kernel sees
///   it. See `docs/architecture-planning.md` decision 6.
/// - `u16`, `u32`, etc. are similarly not `Sample` types. Unsigned arithmetic
///   does not match signal arithmetic.
///
/// # Q-format scaling
///
/// Q-format scaling (Q1.15, Q1.11, Q1.7, etc.) is **not** part of the type.
/// `Complex<i16>` is just a struct of two `i16`s; whether full-scale ±1.0
/// maps to ±32767 (USRP `sc16`, Q1.15) or to ±2048 (BladeRF `SC16_Q11`)
/// depends on the surrounding code. The `q_format` module (Phase 1) provides
/// named constants per radio that encode this.
///
/// # Why no `*Assign` operators?
///
/// The trait deliberately omits `AddAssign`, `SubAssign`, and `MulAssign`.
/// The reason is structural: `num_complex` implements all three for
/// `Complex<T>` only when `T: num_traits::NumAssign`, which transitively
/// requires `DivAssign` and `RemAssign` on `T`. Neither is a meaningful
/// operation on a signed-integer sample (modulo on a Q1.15 sample value is
/// not a signal operation), so widening the trait to admit them would be
/// semantically dubious. Rather than do that, kernels write
/// `x = x + y` instead of `x += y` where they need it. The compiler
/// optimises the two forms identically.
pub trait Sample:
    Copy
    + Clone
    + Debug
    + Default
    + PartialEq
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Neg<Output = Self>
    + Send
    + Sync
    + 'static
{
    /// The additive identity. Equivalent to `Self::default()` for every
    /// type that satisfies `Sample`, but exposed as a `const` so kernels
    /// can use it in const-context (e.g. when zeroing a delay line at
    /// construction).
    const ZERO: Self;
}

impl Sample for f32 {
    const ZERO: Self = 0.0;
}

impl Sample for f64 {
    const ZERO: Self = 0.0;
}

impl Sample for i16 {
    const ZERO: Self = 0;
}

impl Sample for i8 {
    const ZERO: Self = 0;
}

// `Complex<T>` participates as a `Sample` per concrete component type.
//
// A blanket impl `impl<T: Sample> Sample for Complex<T>` would be more
// elegant but cannot be written: `num_complex` implements the `Add`, `Sub`,
// `Mul`, and `Neg` operators on `Complex<T>` only when `T: num_traits::Num`,
// which transitively requires `Div` and `Rem` on `T`. Adding those to the
// `Sample` trait would weaken its semantics (modulo on a Q1.15 sample value
// is meaningless), so the impl is given per concrete type instead. The four
// impls below are exactly the set of complex sample types committed to in
// architecture decision 6.

impl Sample for Complex<f32> {
    const ZERO: Self = Complex::new(0.0, 0.0);
}

impl Sample for Complex<f64> {
    const ZERO: Self = Complex::new(0.0, 0.0);
}

impl Sample for Complex<i16> {
    const ZERO: Self = Complex::new(0, 0);
}

impl Sample for Complex<i8> {
    const ZERO: Self = Complex::new(0, 0);
}

#[cfg(test)]
mod tests {
    use super::*;

    /// `Sample` is satisfied by the four real-valued types committed to in
    /// architecture decision 6.
    #[test]
    fn real_sample_types_satisfy_trait() {
        fn assert_sample<T: Sample>() {}
        assert_sample::<f32>();
        assert_sample::<f64>();
        assert_sample::<i16>();
        assert_sample::<i8>();
    }

    /// `Sample` is satisfied by the four complex-valued types.
    #[test]
    fn complex_sample_types_satisfy_trait() {
        fn assert_sample<T: Sample>() {}
        assert_sample::<Complex<f32>>();
        assert_sample::<Complex<f64>>();
        assert_sample::<Complex<i16>>();
        assert_sample::<Complex<i8>>();
    }

    /// `Complex<u8>` is deliberately not a `Sample`. We can't assert that
    /// it *doesn't* satisfy the trait via a compile-time check without
    /// trait-negative-bounds, but if anyone adds `impl Sample for u8` in
    /// the future this comment is the marker that they need to revisit
    /// architecture decision 6 first.
    #[test]
    fn complex_u8_is_intentionally_not_a_sample() {
        // Intentionally empty. See doc-comment.
    }

    /// `ZERO` round-trips through `Default::default()` for every type.
    ///
    /// The exact-equality comparison on `f32`/`f64` is correct here: we are
    /// asserting that the bit pattern of `ZERO` equals the bit pattern of
    /// `0.0` (`Default::default()` for floats). This is the one numerical
    /// case where `==` on floats is unambiguously the right predicate.
    #[allow(clippy::float_cmp)]
    #[test]
    fn zero_matches_default() {
        assert_eq!(<f32 as Sample>::ZERO, f32::default());
        assert_eq!(<f64 as Sample>::ZERO, f64::default());
        assert_eq!(<i16 as Sample>::ZERO, i16::default());
        assert_eq!(<i8 as Sample>::ZERO, i8::default());
        assert_eq!(<Complex<f32> as Sample>::ZERO, Complex::<f32>::default());
    }
}
