use num_complex::{Complex, ComplexFloat};

/// Multiplies two floating-point complex samples.
///
/// Formula:
///
/// `y = lhs * rhs`
///
/// This is the primitive operation used by a complex mixer. Supplying an NCO
/// sample as `rhs` performs frequency translation.
pub fn complex_multiply<T: ComplexFloat>(lhs: Complex<T>, rhs: Complex<T>) -> Complex<T> {
    lhs * rhs
}

/// Multiplies a floating-point complex sample by the conjugate of another.
///
/// Formula:
///
/// `y = lhs * rhs*`
///
/// This is the primitive normally used for frequency downconversion.
pub fn complex_multiply_conjugate<T: ComplexFloat>(lhs: Complex<T>, rhs: Complex<T>) -> Complex<T> {
    lhs * rhs.conj()
}

/// Right-shifts a signed fixed-point intermediate with round-to-nearest,
/// ties away from zero.
fn round_shift(value: i64, shift: u32) -> i64 {
    if shift == 0 {
        return value;
    }

    let rounding = 1_i64 << (shift - 1);

    if value >= 0 {
        (value + rounding) >> shift
    } else {
        -(((-value) + rounding) >> shift)
    }
}

/// Computes a fixed-point complex product using widened intermediates.
///
/// When `conjugate_rhs` is false:
///
/// `(a + jb)(c + jd) = (ac - bd) + j(ad + bc)`
///
/// When true:
///
/// `(a + jb)(c - jd) = (ac + bd) + j(bc - ad)`
fn fixed_complex_product(
    lhs_re: i64,
    lhs_im: i64,
    rhs_re: i64,
    rhs_im: i64,
    conjugate_rhs: bool,
) -> (i64, i64) {
    if conjugate_rhs {
        (
            lhs_re * rhs_re + lhs_im * rhs_im,
            lhs_im * rhs_re - lhs_re * rhs_im,
        )
    } else {
        (
            lhs_re * rhs_re - lhs_im * rhs_im,
            lhs_re * rhs_im + lhs_im * rhs_re,
        )
    }
}

/// Saturates a widened integer into the `i16` sample range.
fn saturate_i16(value: i64) -> i16 {
    let clamped = value.clamp(i64::from(i16::MIN), i64::from(i16::MAX));

    i16::try_from(clamped).expect("clamped value must fit in i16")
}

/// Saturates a widened integer into the `i8` sample range.
fn saturate_i8(value: i64) -> i8 {
    let clamped = value.clamp(i64::from(i8::MIN), i64::from(i8::MAX));

    i8::try_from(clamped).expect("clamped value must fit in i8")
}

/// Multiplies two fixed-point `Complex<i16>` samples.
///
/// The multiplication is performed using widened `i64` intermediates. `shift`
/// specifies the number of fractional bits removed after multiplication.
///
/// When this is used as a mixer, `shift` normally corresponds to the scale of
/// the NCO operand. For example, a Q1.11 signal multiplied by a Q1.15 NCO uses
/// `shift = 15`, leaving the result in Q1.11.
///
/// Results are rounded to nearest with ties away from zero and saturated into
/// the `i16` range.
///
/// # Panics
///
/// Panics if `shift` is greater than 15.
#[must_use]
pub fn complex_multiply_i16(lhs: Complex<i16>, rhs: Complex<i16>, shift: u32) -> Complex<i16> {
    assert!(
        shift <= 15,
        "fixed-point i16 complex multiply shift must be <= 15"
    );

    let (re, im) = fixed_complex_product(
        i64::from(lhs.re),
        i64::from(lhs.im),
        i64::from(rhs.re),
        i64::from(rhs.im),
        false,
    );

    Complex::new(
        saturate_i16(round_shift(re, shift)),
        saturate_i16(round_shift(im, shift)),
    )
}

/// Multiplies a fixed-point `Complex<i16>` sample by the conjugate of another.
///
/// This is the fixed-point primitive normally used for downconversion. The
/// conjugate is applied algebraically rather than by negating `rhs.im`, which
/// avoids overflow when the imaginary component is `i16::MIN`.
///
/// `shift` has the same meaning as in [`complex_multiply_i16`].
///
/// # Panics
///
/// Panics if `shift` is greater than 15.
#[must_use]
pub fn complex_multiply_conjugate_i16(
    lhs: Complex<i16>,
    rhs: Complex<i16>,
    shift: u32,
) -> Complex<i16> {
    assert!(
        shift <= 15,
        "fixed-point i16 complex multiply shift must be <= 15"
    );

    let (re, im) = fixed_complex_product(
        i64::from(lhs.re),
        i64::from(lhs.im),
        i64::from(rhs.re),
        i64::from(rhs.im),
        true,
    );

    Complex::new(
        saturate_i16(round_shift(re, shift)),
        saturate_i16(round_shift(im, shift)),
    )
}

/// Multiplies two fixed-point `Complex<i8>` samples.
///
/// `shift` specifies the number of fractional bits removed after
/// multiplication. Q1.7 operands normally use `shift = 7`.
///
/// Results are rounded to nearest with ties away from zero and saturated into
/// the `i8` range.
///
/// # Panics
///
/// Panics if `shift` is greater than 7.
#[must_use]
pub fn complex_multiply_i8(lhs: Complex<i8>, rhs: Complex<i8>, shift: u32) -> Complex<i8> {
    assert!(
        shift <= 7,
        "fixed-point i8 complex multiply shift must be <= 7"
    );

    let (re, im) = fixed_complex_product(
        i64::from(lhs.re),
        i64::from(lhs.im),
        i64::from(rhs.re),
        i64::from(rhs.im),
        false,
    );

    Complex::new(
        saturate_i8(round_shift(re, shift)),
        saturate_i8(round_shift(im, shift)),
    )
}

/// Multiplies a fixed-point `Complex<i8>` sample by the conjugate of another.
///
/// This is the fixed-point primitive normally used for downconversion.
///
/// `shift` has the same meaning as in [`complex_multiply_i8`].
///
/// # Panics
///
/// Panics if `shift` is greater than 7.
#[must_use]
pub fn complex_multiply_conjugate_i8(
    lhs: Complex<i8>,
    rhs: Complex<i8>,
    shift: u32,
) -> Complex<i8> {
    assert!(
        shift <= 7,
        "fixed-point i8 complex multiply shift must be <= 7"
    );

    let (re, im) = fixed_complex_product(
        i64::from(lhs.re),
        i64::from(lhs.im),
        i64::from(rhs.re),
        i64::from(rhs.im),
        true,
    );

    Complex::new(
        saturate_i8(round_shift(re, shift)),
        saturate_i8(round_shift(im, shift)),
    )
}

// Floating-point mixer convenience functions

/// Mixes a floating-point signal up in frequency.
pub fn mix_up<T: ComplexFloat>(signal: Complex<T>, nco: Complex<T>) -> Complex<T> {
    complex_multiply(signal, nco)
}

/// Mixes a floating-point signal down in frequency.
pub fn mix_down<T: ComplexFloat>(signal: Complex<T>, nco: Complex<T>) -> Complex<T> {
    complex_multiply_conjugate(signal, nco)
}

// Fixed-point mixer convenience functions

/// Mixes a fixed-point `Complex<i16>` signal up in frequency.
///
/// # Panics
///
/// Panics if `shift` is greater than 15.
#[must_use]
pub fn mix_up_fixed_i16(signal: Complex<i16>, nco: Complex<i16>, shift: u32) -> Complex<i16> {
    complex_multiply_i16(signal, nco, shift)
}

/// Mixes a fixed-point `Complex<i16>` signal down in frequency.
///
/// # Panics
///
/// Panics if `shift` is greater than 15.
#[must_use]
pub fn mix_down_fixed_i16(signal: Complex<i16>, nco: Complex<i16>, shift: u32) -> Complex<i16> {
    complex_multiply_conjugate_i16(signal, nco, shift)
}

/// Mixes a fixed-point `Complex<i8>` signal up in frequency.
///
/// # Panics
///
/// Panics if `shift` is greater than 7.
#[must_use]
pub fn mix_up_fixed_i8(signal: Complex<i8>, nco: Complex<i8>, shift: u32) -> Complex<i8> {
    complex_multiply_i8(signal, nco, shift)
}

/// Mixes a fixed-point `Complex<i8>` signal down in frequency.
///
/// # Panics
///
/// Panics if `shift` is greater than 7.
#[must_use]
pub fn mix_down_fixed_i8(signal: Complex<i8>, nco: Complex<i8>, shift: u32) -> Complex<i8> {
    complex_multiply_conjugate_i8(signal, nco, shift)
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
            .join("mixer")
            .join(filename)
    }

    fn assert_complex_f64_close(actual: &[Complex<f64>], expected: &[Complex<f64>]) {
        let actual_re: Vec<f64> = actual.iter().map(|sample| sample.re).collect();
        let actual_im: Vec<f64> = actual.iter().map(|sample| sample.im).collect();

        let expected_re: Vec<f64> = expected.iter().map(|sample| sample.re).collect();
        let expected_im: Vec<f64> = expected.iter().map(|sample| sample.im).collect();

        assert_close!(actual_re, expected_re, rtol = 1e-12, atol = 1e-15);
        assert_close!(actual_im, expected_im, rtol = 1e-12, atol = 1e-15);
    }

    fn assert_complex_f32_close(actual: &[Complex<f32>], expected: &[Complex<f32>]) {
        let actual_re: Vec<f32> = actual.iter().map(|sample| sample.re).collect();
        let actual_im: Vec<f32> = actual.iter().map(|sample| sample.im).collect();

        let expected_re: Vec<f32> = expected.iter().map(|sample| sample.re).collect();
        let expected_im: Vec<f32> = expected.iter().map(|sample| sample.im).collect();

        assert_close!(actual_re, expected_re, rtol = 1e-6, atol = 1e-7);
        assert_close!(actual_im, expected_im, rtol = 1e-6, atol = 1e-7);
    }

    #[test]
    fn test_complex_multiply_f64_matches_numpy_oracle() {
        for n in [8usize, 16, 64] {
            let lhs = npy::load_complex_f64(&vector_path(&format!("input_f64_{n}.npy")))
                .expect("failed to load f64 mixer input vector");

            let rhs = npy::load_complex_f64(&vector_path(&format!("nco_f64_{n}.npy")))
                .expect("failed to load f64 mixer NCO vector");

            let expected = npy::load_complex_f64(&vector_path(&format!("multiply_f64_{n}.npy")))
                .expect("failed to load f64 complex multiply reference vector");

            let actual: Vec<Complex<f64>> = lhs
                .iter()
                .zip(rhs.iter())
                .map(|(&a, &b)| complex_multiply(a, b))
                .collect();

            assert_complex_f64_close(&actual, &expected);
        }
    }

    #[test]
    fn test_complex_multiply_conjugate_f64_matches_numpy_oracle() {
        for n in [8usize, 16, 64] {
            let lhs = npy::load_complex_f64(&vector_path(&format!("input_f64_{n}.npy")))
                .expect("failed to load f64 mixer input vector");

            let rhs = npy::load_complex_f64(&vector_path(&format!("nco_f64_{n}.npy")))
                .expect("failed to load f64 mixer NCO vector");

            let expected =
                npy::load_complex_f64(&vector_path(&format!("multiply_conj_f64_{n}.npy")))
                    .expect("failed to load f64 conjugate multiply reference vector");

            let actual: Vec<Complex<f64>> = lhs
                .iter()
                .zip(rhs.iter())
                .map(|(&a, &b)| complex_multiply_conjugate(a, b))
                .collect();

            assert_complex_f64_close(&actual, &expected);
        }
    }

    #[test]
    fn test_complex_multiply_f32_matches_numpy_oracle() {
        for n in [8usize, 16, 64] {
            let lhs = npy::load_complex_f32(&vector_path(&format!("input_f32_{n}.npy")))
                .expect("failed to load f32 mixer input vector");

            let rhs = npy::load_complex_f32(&vector_path(&format!("nco_f32_{n}.npy")))
                .expect("failed to load f32 mixer NCO vector");

            let expected = npy::load_complex_f32(&vector_path(&format!("multiply_f32_{n}.npy")))
                .expect("failed to load f32 complex multiply reference vector");

            let actual: Vec<Complex<f32>> = lhs
                .iter()
                .zip(rhs.iter())
                .map(|(&a, &b)| complex_multiply(a, b))
                .collect();

            assert_complex_f32_close(&actual, &expected);
        }
    }

    #[test]
    fn test_complex_multiply_i16_matches_numpy_oracle_bit_exact() {
        const SHIFT: u32 = 15;

        for n in [8usize, 16, 64] {
            let lhs = npy::load_complex_i16(&vector_path(&format!("input_i16_{n}.npy")))
                .expect("failed to load i16 mixer input vector");

            let rhs = npy::load_complex_i16(&vector_path(&format!("nco_i16_{n}.npy")))
                .expect("failed to load i16 mixer NCO vector");

            let expected = npy::load_complex_i16(&vector_path(&format!("multiply_i16_{n}.npy")))
                .expect("failed to load i16 complex multiply reference vector");

            let actual: Vec<Complex<i16>> = lhs
                .iter()
                .zip(rhs.iter())
                .map(|(&a, &b)| complex_multiply_i16(a, b, SHIFT))
                .collect();

            assert_bit_exact!(actual, expected);
        }
    }

    #[test]
    fn test_complex_multiply_conjugate_i16_matches_numpy_oracle_bit_exact() {
        const SHIFT: u32 = 15;

        for n in [8usize, 16, 64] {
            let lhs = npy::load_complex_i16(&vector_path(&format!("input_i16_{n}.npy")))
                .expect("failed to load i16 mixer input vector");

            let rhs = npy::load_complex_i16(&vector_path(&format!("nco_i16_{n}.npy")))
                .expect("failed to load i16 mixer NCO vector");

            let expected =
                npy::load_complex_i16(&vector_path(&format!("multiply_conj_i16_{n}.npy")))
                    .expect("failed to load i16 conjugate multiply reference vector");

            let actual: Vec<Complex<i16>> = lhs
                .iter()
                .zip(rhs.iter())
                .map(|(&a, &b)| complex_multiply_conjugate_i16(a, b, SHIFT))
                .collect();

            assert_bit_exact!(actual, expected);
        }
    }

    #[test]
    fn test_complex_multiply_i8_matches_numpy_oracle_bit_exact() {
        const SHIFT: u32 = 7;

        for n in [8usize, 16, 64] {
            let lhs = npy::load_complex_i8(&vector_path(&format!("input_i8_{n}.npy")))
                .expect("failed to load i8 mixer input vector");

            let rhs = npy::load_complex_i8(&vector_path(&format!("nco_i8_{n}.npy")))
                .expect("failed to load i8 mixer NCO vector");

            let expected = npy::load_complex_i8(&vector_path(&format!("multiply_i8_{n}.npy")))
                .expect("failed to load i8 complex multiply reference vector");

            let actual: Vec<Complex<i8>> = lhs
                .iter()
                .zip(rhs.iter())
                .map(|(&a, &b)| complex_multiply_i8(a, b, SHIFT))
                .collect();

            assert_bit_exact!(actual, expected);
        }
    }

    #[test]
    fn test_complex_multiply_conjugate_i8_matches_numpy_oracle_bit_exact() {
        const SHIFT: u32 = 7;

        for n in [8usize, 16, 64] {
            let lhs = npy::load_complex_i8(&vector_path(&format!("input_i8_{n}.npy")))
                .expect("failed to load i8 mixer input vector");

            let rhs = npy::load_complex_i8(&vector_path(&format!("nco_i8_{n}.npy")))
                .expect("failed to load i8 mixer NCO vector");

            let expected = npy::load_complex_i8(&vector_path(&format!("multiply_conj_i8_{n}.npy")))
                .expect("failed to load i8 conjugate multiply reference vector");

            let actual: Vec<Complex<i8>> = lhs
                .iter()
                .zip(rhs.iter())
                .map(|(&a, &b)| complex_multiply_conjugate_i8(a, b, SHIFT))
                .collect();

            assert_bit_exact!(actual, expected);
        }
    }

    #[test]
    fn test_i16_complex_multiply_saturates() {
        let lhs = Complex::new(i16::MAX, i16::MAX);
        let rhs = Complex::new(i16::MAX, i16::MAX);

        let actual = complex_multiply_i16(lhs, rhs, 15);

        assert_eq!(actual.re, 0);
        assert_eq!(actual.im, i16::MAX);
    }

    #[test]
    fn test_i8_complex_multiply_saturates() {
        let lhs = Complex::new(i8::MAX, i8::MAX);
        let rhs = Complex::new(i8::MAX, i8::MAX);

        let actual = complex_multiply_i8(lhs, rhs, 7);

        assert_eq!(actual.re, 0);
        assert_eq!(actual.im, i8::MAX);
    }
}
