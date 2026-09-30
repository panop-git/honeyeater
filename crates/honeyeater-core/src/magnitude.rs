//! Complex magnitude and power calculation.
//!
//! Floating-point inputs return floating-point magnitude and power.
//! Fixed-point `Complex<i16>` and `Complex<i8>` inputs use widened integer
//! arithmetic and return raw unsigned counts in `u32`.
//!
//! For fixed-point samples, magnitude retains the input scale while power has
//! the square of the input scale. For example, if a `Complex<i16>` sample is
//! interpreted as Q1.15, divide its magnitude by `2^15` to obtain a normalised
//! floating-point magnitude, or divide its power by `2^30` to obtain
//! normalised power.

use crate::Sample;
use num_complex::Complex;

/// Component types supported by the complex magnitude/power primitives.
///
/// Floating-point component types return their corresponding floating-point
/// scalar type. Fixed-point integer component types return `u32` so that the
/// full `Complex<i16>` power range is representable without overflow.
pub trait MagnitudeSample: Sample {
    /// Scalar output type produced by magnitude and power calculations.
    type Output: Copy + Default;

    /// Computes squared complex magnitude.
    fn power(sample: Complex<Self>) -> Self::Output;

    /// Computes complex magnitude.
    fn magnitude(sample: Complex<Self>) -> Self::Output;
}

impl MagnitudeSample for f32 {
    type Output = f32;

    fn power(sample: Complex<Self>) -> Self::Output {
        sample.re * sample.re + sample.im * sample.im
    }

    fn magnitude(sample: Complex<Self>) -> Self::Output {
        Self::power(sample).sqrt()
    }
}

impl MagnitudeSample for f64 {
    type Output = f64;

    fn power(sample: Complex<Self>) -> Self::Output {
        sample.re * sample.re + sample.im * sample.im
    }

    fn magnitude(sample: Complex<Self>) -> Self::Output {
        Self::power(sample).sqrt()
    }
}

impl MagnitudeSample for i16 {
    type Output = u32;

    fn power(sample: Complex<Self>) -> Self::Output {
        let re = i64::from(sample.re);
        let im = i64::from(sample.im);
        let power = re * re + im * im;

        u32::try_from(power).expect("Complex<i16> power must fit in u32")
    }

    fn magnitude(sample: Complex<Self>) -> Self::Output {
        Self::power(sample).isqrt()
    }
}

impl MagnitudeSample for i8 {
    type Output = u32;

    fn power(sample: Complex<Self>) -> Self::Output {
        let re = i32::from(sample.re);
        let im = i32::from(sample.im);
        let power = re * re + im * im;

        u32::try_from(power).expect("Complex<i8> power must fit in u32")
    }

    fn magnitude(sample: Complex<Self>) -> Self::Output {
        Self::power(sample).isqrt()
    }
}

/// Computes the squared magnitude of one complex sample.
///
/// For floating-point samples this is `re² + im²`. For fixed-point integer
/// samples the calculation is widened before multiplication and returned as an
/// unsigned raw count.
#[must_use]
pub fn complex_power<T: MagnitudeSample>(sample: Complex<T>) -> T::Output {
    T::power(sample)
}

/// Computes the magnitude of one complex sample.
///
/// Fixed-point integer magnitude uses exact integer square root and therefore
/// rounds downward to the nearest integer count.
#[must_use]
pub fn complex_magnitude<T: MagnitudeSample>(sample: Complex<T>) -> T::Output {
    T::magnitude(sample)
}

/// Computes squared magnitude for a block of complex samples.
///
/// No allocation is performed.
///
/// # Panics
///
/// Panics if `input` and `output` have different lengths.
pub fn complex_powers<T: MagnitudeSample>(
    input: &[Complex<T>],
    output: &mut [T::Output],
) {
    assert_eq!(
        input.len(),
        output.len(),
        "magnitude input and output lengths must match"
    );

    for (&sample, output_sample) in input.iter().zip(output.iter_mut()) {
        *output_sample = complex_power(sample);
    }
}

/// Computes magnitude for a block of complex samples.
///
/// No allocation is performed.
///
/// # Panics
///
/// Panics if `input` and `output` have different lengths.
pub fn complex_magnitudes<T: MagnitudeSample>(
    input: &[Complex<T>],
    output: &mut [T::Output],
) {
    assert_eq!(
        input.len(),
        output.len(),
        "magnitude input and output lengths must match"
    );

    for (&sample, output_sample) in input.iter().zip(output.iter_mut()) {
        *output_sample = complex_magnitude(sample);
    }
}

/// Computes squared magnitude into a newly allocated output vector.
///
/// Allocates a fresh output buffer on every call. Suitable for offline
/// analysis and one-shot processing. Not suitable for hard real-time
/// streaming pipelines where a missed buffer drops samples.
#[must_use]
pub fn complex_powers_owned<T: MagnitudeSample>(
    input: &[Complex<T>],
) -> Vec<T::Output> {
    let mut output = vec![T::Output::default(); input.len()];
    complex_powers(input, &mut output);
    output
}

/// Computes magnitude into a newly allocated output vector.
///
/// Allocates a fresh output buffer on every call. Suitable for offline
/// analysis and one-shot processing. Not suitable for hard real-time
/// streaming pipelines where a missed buffer drops samples.
#[must_use]
pub fn complex_magnitudes_owned<T: MagnitudeSample>(
    input: &[Complex<T>],
) -> Vec<T::Output> {
    let mut output = vec![T::Output::default(); input.len()];
    complex_magnitudes(input, &mut output);
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use honeyeater_test::{assert_bit_exact, assert_close, npy};
    use std::path::{Path, PathBuf};

    fn vector_path(filename: &str) -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("vectors")
            .join("magnitude")
            .join(filename)
    }

    #[test]
    fn test_magnitude_f64_matches_numpy_oracle() {
        let input = npy::load_complex_f64(&vector_path("input_f64.npy"))
            .expect("failed to load f64 magnitude input");

        let expected_power = npy::load_f64(&vector_path("power_f64.npy"))
            .expect("failed to load f64 power oracle");

        let expected_magnitude =
            npy::load_f64(&vector_path("magnitude_f64.npy"))
                .expect("failed to load f64 magnitude oracle");

        let mut actual_power = vec![0.0_f64; input.len()];
        let mut actual_magnitude = vec![0.0_f64; input.len()];

        complex_powers(&input, &mut actual_power);
        complex_magnitudes(&input, &mut actual_magnitude);

        assert_close!(
            actual_power,
            expected_power,
            rtol = 1e-12,
            atol = 1e-15
        );

        assert_close!(
            actual_magnitude,
            expected_magnitude,
            rtol = 1e-12,
            atol = 1e-15
        );
    }

    #[test]
    fn test_magnitude_f32_matches_numpy_oracle() {
        let input = npy::load_complex_f32(&vector_path("input_f32.npy"))
            .expect("failed to load f32 magnitude input");

        let expected_power = npy::load_f32(&vector_path("power_f32.npy"))
            .expect("failed to load f32 power oracle");

        let expected_magnitude =
            npy::load_f32(&vector_path("magnitude_f32.npy"))
                .expect("failed to load f32 magnitude oracle");

        let mut actual_power = vec![0.0_f32; input.len()];
        let mut actual_magnitude = vec![0.0_f32; input.len()];

        complex_powers(&input, &mut actual_power);
        complex_magnitudes(&input, &mut actual_magnitude);

        assert_close!(
            actual_power,
            expected_power,
            rtol = 1e-6,
            atol = 1e-7
        );

        assert_close!(
            actual_magnitude,
            expected_magnitude,
            rtol = 1e-6,
            atol = 1e-7
        );
    }

    #[test]
    fn test_magnitude_i16_matches_integer_oracle_bit_exact() {
        let input = npy::load_complex_i16(&vector_path("input_i16.npy"))
            .expect("failed to load i16 magnitude input");

        let expected_power = npy::load_u32(&vector_path("power_i16.npy"))
            .expect("failed to load i16 power oracle");

        let expected_magnitude =
            npy::load_u32(&vector_path("magnitude_i16.npy"))
                .expect("failed to load i16 magnitude oracle");

        let mut actual_power = vec![0_u32; input.len()];
        let mut actual_magnitude = vec![0_u32; input.len()];

        complex_powers(&input, &mut actual_power);
        complex_magnitudes(&input, &mut actual_magnitude);

        assert_bit_exact!(actual_power, expected_power);
        assert_bit_exact!(actual_magnitude, expected_magnitude);
    }

    #[test]
    fn test_magnitude_i8_matches_integer_oracle_bit_exact() {
        let input = npy::load_complex_i8(&vector_path("input_i8.npy"))
            .expect("failed to load i8 magnitude input");

        let expected_power = npy::load_u32(&vector_path("power_i8.npy"))
            .expect("failed to load i8 power oracle");

        let expected_magnitude =
            npy::load_u32(&vector_path("magnitude_i8.npy"))
                .expect("failed to load i8 magnitude oracle");

        let mut actual_power = vec![0_u32; input.len()];
        let mut actual_magnitude = vec![0_u32; input.len()];

        complex_powers(&input, &mut actual_power);
        complex_magnitudes(&input, &mut actual_magnitude);

        assert_bit_exact!(actual_power, expected_power);
        assert_bit_exact!(actual_magnitude, expected_magnitude);
    }

    #[test]
    fn test_fixed_point_three_four_five_triangle() {
        let sample = Complex::new(3_i16, 4_i16);

        assert_eq!(complex_power(sample), 25);
        assert_eq!(complex_magnitude(sample), 5);
    }

    #[test]
    fn test_i16_full_scale_power_does_not_overflow() {
        let sample = Complex::new(i16::MIN, i16::MIN);

        assert_eq!(complex_power(sample), 2_147_483_648);
        assert_eq!(complex_magnitude(sample), 46_340);
    }

    #[test]
    fn test_owned_and_borrowed_forms_match() {
        let input = [
            Complex::new(3_i16, 4_i16),
            Complex::new(-5_i16, 12_i16),
        ];

        let mut borrowed = [0_u32; 2];
        complex_magnitudes(&input, &mut borrowed);

        let owned = complex_magnitudes_owned(&input);

        assert_bit_exact!(owned, borrowed);
    }
}