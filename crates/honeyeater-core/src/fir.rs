use num_complex::Complex;

use crate::Sample;

/// A stateful finite impulse response filter.
///
/// Coefficients are stored in forward FIR order:
///
/// - `coefficients[0]` multiplies the newest input sample.
/// - `coefficients[1]` multiplies the previous input sample.
/// - and so on.
///
/// The filter maintains its delay-line state between calls to [`FirFilter::process`].
///
/// For fixed-point filters, the coefficient Q-format is supplied at
/// construction. The input sample Q-format is preserved by the filtering
/// operation.
pub struct FirFilter<T: Sample> {
    coefficients: Vec<T>,
    delay_line: Vec<T>,
    write_index: usize,
    coefficient_fractional_bits: u32,
}

impl<T: Sample> FirFilter<T> {
    fn from_coefficients(coefficients: &[T], coefficient_fractional_bits: u32) -> Self {
        assert!(
            !coefficients.is_empty(),
            "FIR filter requires at least one coefficient"
        );

        Self {
            coefficients: coefficients.to_vec(),
            delay_line: vec![T::ZERO; coefficients.len()],
            write_index: 0,
            coefficient_fractional_bits,
        }
    }

    /// Clears the FIR delay line without changing the coefficients.
    pub fn reset(&mut self) {
        self.delay_line.fill(T::ZERO);
        self.write_index = 0;
    }

    fn push_sample(&mut self, input: T) -> usize {
        let newest_index = self.write_index;

        self.delay_line[newest_index] = input;
        self.write_index = (self.write_index + 1) % self.delay_line.len();

        newest_index
    }

    fn delayed_sample(&self, newest_index: usize, tap_index: usize) -> T {
        let len = self.delay_line.len();
        let index = (newest_index + len - tap_index) % len;

        self.delay_line[index]
    }
}

/// Executes one floating-point FIR sample.
///
/// This helper is private so the direct `T * T` arithmetic is only exposed
/// through the floating-point implementations below. Fixed-point execution
/// uses widened integer accumulators instead.
fn process_float_sample<T: Sample>(filter: &mut FirFilter<T>, input: T) -> T {
    let newest_index = filter.push_sample(input);
    let mut accumulator = T::ZERO;

    for (tap_index, &coefficient) in filter.coefficients.iter().enumerate() {
        let sample = filter.delayed_sample(newest_index, tap_index);
        accumulator = accumulator + sample * coefficient;
    }

    accumulator
}

fn process_float_block<T: Sample>(filter: &mut FirFilter<T>, input: &[T], output: &mut [T]) {
    assert_eq!(
        input.len(),
        output.len(),
        "FIR input and output lengths must match"
    );

    for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
        *output_sample = process_float_sample(filter, input_sample);
    }
}

macro_rules! impl_float_fir {
    ($type:ty) => {
        impl FirFilter<$type> {
            /// Creates a floating-point FIR filter.
            ///
            /// # Panics
            ///
            /// Panics if `coefficients` is empty.
            #[must_use]
            pub fn new(coefficients: &[$type]) -> Self {
                Self::from_coefficients(coefficients, 0)
            }

            /// Processes one sample through the FIR filter.
            #[must_use]
            pub fn process_sample(&mut self, input: $type) -> $type {
                process_float_sample(self, input)
            }

            /// Processes a block while preserving filter state between calls.
            ///
            /// No allocation is performed.
            ///
            /// # Panics
            ///
            /// Panics if `input` and `output` have different lengths.
            pub fn process(&mut self, input: &[$type], output: &mut [$type]) {
                process_float_block(self, input, output);
            }

            /// Processes a block into a newly allocated output vector while
            /// preserving filter state.
            ///
            /// Allocates a fresh output buffer on every call. Suitable for offline
            /// analysis and one-shot processing. Not suitable for hard real-time
            /// streaming pipelines where a missed buffer drops samples.
            #[must_use]
            pub fn process_owned(&mut self, input: &[$type]) -> Vec<$type> {
                let mut output = vec![<$type as Sample>::ZERO; input.len()];
                self.process(input, &mut output);
                output
            }
        }
    };
}

impl_float_fir!(f32);
impl_float_fir!(f64);
impl_float_fir!(Complex<f32>);
impl_float_fir!(Complex<f64>);

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

fn saturate_i16(value: i64) -> i16 {
    let clamped = value.clamp(i64::from(i16::MIN), i64::from(i16::MAX));

    i16::try_from(clamped).expect("clamped value must fit in i16")
}

fn saturate_i8(value: i64) -> i8 {
    let clamped = value.clamp(i64::from(i8::MIN), i64::from(i8::MAX));

    i8::try_from(clamped).expect("clamped value must fit in i8")
}

impl FirFilter<Complex<i16>> {
    /// Creates a fixed-point `Complex<i16>` FIR filter.
    ///
    /// `coefficient_fractional_bits` specifies the Q-format of the filter
    /// coefficients. For example:
    ///
    /// - Q1.15 coefficients use `15`.
    /// - Q1.11 coefficients use `11`.
    ///
    /// The input sample scale is preserved at the output.
    ///
    /// Real-valued FIR coefficients are represented as complex values with an
    /// imaginary component of zero.
    ///
    /// # Panics
    ///
    /// Panics if `coefficients` is empty or if
    /// `coefficient_fractional_bits > 15`.
    #[must_use]
    pub fn new(coefficients: &[Complex<i16>], coefficient_fractional_bits: u32) -> Self {
        assert!(
            coefficient_fractional_bits <= 15,
            "i16 FIR coefficient fractional bits must be <= 15"
        );

        Self::from_coefficients(coefficients, coefficient_fractional_bits)
    }

    /// Processes one fixed-point `Complex<i16>` sample.
    ///
    /// Multiplication and accumulation use widened `i64` intermediates.
    /// Rounding occurs once after the complete multiply-accumulate operation,
    /// followed by saturation into the `i16` output range.
    #[must_use]
    pub fn process_sample(&mut self, input: Complex<i16>) -> Complex<i16> {
        let newest_index = self.push_sample(input);

        let mut accumulator_re = 0_i64;
        let mut accumulator_im = 0_i64;

        for (tap_index, &coefficient) in self.coefficients.iter().enumerate() {
            let sample = self.delayed_sample(newest_index, tap_index);

            let sample_re = i64::from(sample.re);
            let sample_im = i64::from(sample.im);
            let coefficient_re = i64::from(coefficient.re);
            let coefficient_im = i64::from(coefficient.im);

            accumulator_re += sample_re * coefficient_re - sample_im * coefficient_im;

            accumulator_im += sample_re * coefficient_im + sample_im * coefficient_re;
        }

        Complex::new(
            saturate_i16(round_shift(
                accumulator_re,
                self.coefficient_fractional_bits,
            )),
            saturate_i16(round_shift(
                accumulator_im,
                self.coefficient_fractional_bits,
            )),
        )
    }

    /// Processes a fixed-point `Complex<i16>` block while preserving state.
    ///
    /// No allocation is performed.
    ///
    /// # Panics
    ///
    /// Panics if `input` and `output` have different lengths.
    pub fn process(&mut self, input: &[Complex<i16>], output: &mut [Complex<i16>]) {
        assert_eq!(
            input.len(),
            output.len(),
            "FIR input and output lengths must match"
        );

        for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
            *output_sample = self.process_sample(input_sample);
        }
    }

    /// Processes a fixed-point block into a newly allocated output vector while
    /// preserving filter state.
    ///
    /// Allocates a fresh output buffer on every call. Suitable for offline
    /// analysis and one-shot processing. Not suitable for hard real-time
    /// streaming pipelines where a missed buffer drops samples.
    #[must_use]
    pub fn process_owned(
        &mut self,
        input: &[Complex<i16>],
    ) -> Vec<Complex<i16>> {
        let mut output = vec![Complex::new(0_i16, 0_i16); input.len()];
        self.process(input, &mut output);
        output
    }
}

impl FirFilter<Complex<i8>> {
    /// Creates a fixed-point `Complex<i8>` FIR filter.
    ///
    /// Q1.7 coefficients use `coefficient_fractional_bits = 7`.
    ///
    /// The input sample scale is preserved at the output.
    ///
    /// # Panics
    ///
    /// Panics if `coefficients` is empty or if
    /// `coefficient_fractional_bits > 7`.
    #[must_use]
    pub fn new(coefficients: &[Complex<i8>], coefficient_fractional_bits: u32) -> Self {
        assert!(
            coefficient_fractional_bits <= 7,
            "i8 FIR coefficient fractional bits must be <= 7"
        );

        Self::from_coefficients(coefficients, coefficient_fractional_bits)
    }

    /// Processes one fixed-point `Complex<i8>` sample.
    ///
    /// Multiplication and accumulation use widened `i64` intermediates,
    /// followed by rounding and saturation.
    #[must_use]
    pub fn process_sample(&mut self, input: Complex<i8>) -> Complex<i8> {
        let newest_index = self.push_sample(input);

        let mut accumulator_re = 0_i64;
        let mut accumulator_im = 0_i64;

        for (tap_index, &coefficient) in self.coefficients.iter().enumerate() {
            let sample = self.delayed_sample(newest_index, tap_index);

            let sample_re = i64::from(sample.re);
            let sample_im = i64::from(sample.im);
            let coefficient_re = i64::from(coefficient.re);
            let coefficient_im = i64::from(coefficient.im);

            accumulator_re += sample_re * coefficient_re - sample_im * coefficient_im;

            accumulator_im += sample_re * coefficient_im + sample_im * coefficient_re;
        }

        Complex::new(
            saturate_i8(round_shift(
                accumulator_re,
                self.coefficient_fractional_bits,
            )),
            saturate_i8(round_shift(
                accumulator_im,
                self.coefficient_fractional_bits,
            )),
        )
    }

    /// Processes a fixed-point `Complex<i8>` block while preserving state.
    ///
    /// No allocation is performed.
    ///
    /// # Panics
    ///
    /// Panics if `input` and `output` have different lengths.
    pub fn process(&mut self, input: &[Complex<i8>], output: &mut [Complex<i8>]) {
        assert_eq!(
            input.len(),
            output.len(),
            "FIR input and output lengths must match"
        );

        for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
            *output_sample = self.process_sample(input_sample);
        }
    }

    /// Processes a fixed-point block into a newly allocated output vector while
    /// preserving filter state.
    ///
    /// Allocates a fresh output buffer on every call. Suitable for offline
    /// analysis and one-shot processing. Not suitable for hard real-time
    /// streaming pipelines where a missed buffer drops samples.
    #[must_use]
    pub fn process_owned(
        &mut self,
        input: &[Complex<i8>],
    ) -> Vec<Complex<i8>> {
        let mut output = vec![Complex::new(0_i8, 0_i8); input.len()];
        self.process(input, &mut output);
        output
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use honeyeater_test::{assert_close, assert_snr_db, npy};
    use std::path::PathBuf;

    fn vector_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("vectors");
        path.push("fir");
        path.push(filename);
        path
    }

    fn split_complex_f64(values: &[Complex<f64>]) -> (Vec<f64>, Vec<f64>) {
        values.iter().map(|value| (value.re, value.im)).unzip()
    }

    fn split_complex_f32(values: &[Complex<f32>]) -> (Vec<f32>, Vec<f32>) {
        values.iter().map(|value| (value.re, value.im)).unzip()
    }

    #[allow(clippy::cast_possible_truncation)]
    fn quantize_i16(value: f64, fractional_bits: u32) -> i16 {
        let scale = f64::from(1_u32 << fractional_bits);

        let scaled = (value * scale)
            .round()
            .clamp(f64::from(i16::MIN), f64::from(i16::MAX));

        scaled as i16
    }

    #[allow(clippy::cast_possible_truncation)]
    fn quantize_i8(value: f64, fractional_bits: u32) -> i8 {
        let scale = f64::from(1_u32 << fractional_bits);

        let scaled = (value * scale)
            .round()
            .clamp(f64::from(i8::MIN), f64::from(i8::MAX));

        scaled as i8
    }

    fn quantize_complex_i16(values: &[Complex<f64>], fractional_bits: u32) -> Vec<Complex<i16>> {
        values
            .iter()
            .map(|value| {
                Complex::new(
                    quantize_i16(value.re, fractional_bits),
                    quantize_i16(value.im, fractional_bits),
                )
            })
            .collect()
    }

    fn quantize_complex_i8(values: &[Complex<f64>], fractional_bits: u32) -> Vec<Complex<i8>> {
        values
            .iter()
            .map(|value| {
                Complex::new(
                    quantize_i8(value.re, fractional_bits),
                    quantize_i8(value.im, fractional_bits),
                )
            })
            .collect()
    }

    fn quantize_taps_i16(taps: &[f64], fractional_bits: u32) -> Vec<Complex<i16>> {
        taps.iter()
            .map(|&tap| Complex::new(quantize_i16(tap, fractional_bits), 0))
            .collect()
    }

    fn quantize_taps_i8(taps: &[f64], fractional_bits: u32) -> Vec<Complex<i8>> {
        taps.iter()
            .map(|&tap| Complex::new(quantize_i8(tap, fractional_bits), 0))
            .collect()
    }

    fn dequantize_i16(values: &[Complex<i16>], fractional_bits: u32) -> Vec<Complex<f64>> {
        let scale = f64::from(1_u32 << fractional_bits);

        values
            .iter()
            .map(|value| Complex::new(f64::from(value.re) / scale, f64::from(value.im) / scale))
            .collect()
    }

    fn dequantize_i8(values: &[Complex<i8>], fractional_bits: u32) -> Vec<Complex<f64>> {
        let scale = f64::from(1_u32 << fractional_bits);

        values
            .iter()
            .map(|value| Complex::new(f64::from(value.re) / scale, f64::from(value.im) / scale))
            .collect()
    }

    /// Conservative component-wise error bound for a real-coefficient FIR
    /// where both input and coefficients have been quantised to the same Q
    /// format.
    ///
    /// For one product:
    ///
    /// |(x + ex)(h + eh) - xh|
    /// <= |x|*|eh| + |h|*|ex| + |ex|*|eh|
    ///
    /// The bound is summed across all taps and one half-LSB is added for the
    /// final accumulator rounding.
    fn quantization_bound(input: &[Complex<f64>], taps: &[f64], fractional_bits: u32) -> f64 {
        let scale = f64::from(1_u32 << fractional_bits);
        let half_lsb = 0.5 / scale;

        let max_input = input.iter().fold(0.0_f64, |current, sample| {
            current.max(sample.re.abs()).max(sample.im.abs())
        });

        let max_tap = taps
            .iter()
            .fold(0.0_f64, |current, tap| current.max(tap.abs()));

        let tap_count =
            f64::from(u32::try_from(taps.len()).expect("FIR tap count must fit in u32"));

        tap_count * (max_input * half_lsb + max_tap * half_lsb + half_lsb * half_lsb) + half_lsb
    }

    #[test]
    fn test_fir_f64_matches_scipy_oracle() {
        let taps = npy::load_f64(&vector_path("taps_f64.npy"))
            .expect("failed to load FIR coefficient vector");

        let coefficients: Vec<Complex<f64>> =
            taps.iter().map(|&tap| Complex::new(tap, 0.0)).collect();

        for n in [8usize, 16, 64] {
            let input = npy::load_complex_f64(&vector_path(&format!("input_f64_{n}.npy")))
                .expect("failed to load FIR input vector");

            let expected = npy::load_complex_f64(&vector_path(&format!("output_f64_{n}.npy")))
                .expect("failed to load FIR reference vector");

            let mut filter = FirFilter::<Complex<f64>>::new(&coefficients);
            let mut actual = vec![Complex::default(); input.len()];

            // Split processing deliberately to verify state continuity between
            // successive streaming calls.
            let split = input.len() / 3;

            filter.process(&input[..split], &mut actual[..split]);
            filter.process(&input[split..], &mut actual[split..]);

            let (actual_re, actual_im) = split_complex_f64(&actual);
            let (expected_re, expected_im) = split_complex_f64(&expected);

            assert_snr_db!(actual_re, expected_re, min_db = 100.0);
            assert_snr_db!(actual_im, expected_im, min_db = 100.0);
        }
    }

    #[test]
    fn test_fir_f32_matches_scipy_oracle() {
        let taps = npy::load_f32(&vector_path("taps_f32.npy"))
            .expect("failed to load FIR coefficient vector");

        let coefficients: Vec<Complex<f32>> =
            taps.iter().map(|&tap| Complex::new(tap, 0.0)).collect();

        for n in [8usize, 16, 64] {
            let input = npy::load_complex_f32(&vector_path(&format!("input_f32_{n}.npy")))
                .expect("failed to load FIR input vector");

            let expected = npy::load_complex_f32(&vector_path(&format!("output_f32_{n}.npy")))
                .expect("failed to load FIR reference vector");

            let mut filter = FirFilter::<Complex<f32>>::new(&coefficients);
            let mut actual = vec![Complex::default(); input.len()];

            filter.process(&input, &mut actual);

            let (actual_re, actual_im) = split_complex_f32(&actual);
            let (expected_re, expected_im) = split_complex_f32(&expected);

            assert_snr_db!(actual_re, expected_re, min_db = 60.0);
            assert_snr_db!(actual_im, expected_im, min_db = 60.0);
        }
    }

    #[test]
    fn test_fir_i16_within_q15_quantization_bound() {
        const FRACTIONAL_BITS: u32 = 15;

        let taps = npy::load_f64(&vector_path("taps_f64.npy"))
            .expect("failed to load FIR coefficient vector");

        let input = npy::load_complex_f64(&vector_path("input_f64_64.npy"))
            .expect("failed to load FIR input vector");

        let expected = npy::load_complex_f64(&vector_path("output_f64_64.npy"))
            .expect("failed to load FIR reference vector");

        let fixed_input = quantize_complex_i16(&input, FRACTIONAL_BITS);
        let fixed_taps = quantize_taps_i16(&taps, FRACTIONAL_BITS);

        let mut filter = FirFilter::<Complex<i16>>::new(&fixed_taps, FRACTIONAL_BITS);

        let mut fixed_output = vec![Complex::default(); fixed_input.len()];
        filter.process(&fixed_input, &mut fixed_output);

        let actual = dequantize_i16(&fixed_output, FRACTIONAL_BITS);
        let bound = quantization_bound(&input, &taps, FRACTIONAL_BITS);

        let (actual_re, actual_im) = split_complex_f64(&actual);
        let (expected_re, expected_im) = split_complex_f64(&expected);

        assert_close!(actual_re, expected_re, rtol = 0.0, atol = bound);
        assert_close!(actual_im, expected_im, rtol = 0.0, atol = bound);
    }

    #[test]
    fn test_fir_i8_within_q7_quantization_bound() {
        const FRACTIONAL_BITS: u32 = 7;

        let taps = npy::load_f64(&vector_path("taps_f64.npy"))
            .expect("failed to load FIR coefficient vector");

        let input = npy::load_complex_f64(&vector_path("input_f64_64.npy"))
            .expect("failed to load FIR input vector");

        let expected = npy::load_complex_f64(&vector_path("output_f64_64.npy"))
            .expect("failed to load FIR reference vector");

        let fixed_input = quantize_complex_i8(&input, FRACTIONAL_BITS);
        let fixed_taps = quantize_taps_i8(&taps, FRACTIONAL_BITS);

        let mut filter = FirFilter::<Complex<i8>>::new(&fixed_taps, FRACTIONAL_BITS);

        let mut fixed_output = vec![Complex::default(); fixed_input.len()];
        filter.process(&fixed_input, &mut fixed_output);

        let actual = dequantize_i8(&fixed_output, FRACTIONAL_BITS);
        let bound = quantization_bound(&input, &taps, FRACTIONAL_BITS);

        let (actual_re, actual_im) = split_complex_f64(&actual);
        let (expected_re, expected_im) = split_complex_f64(&expected);

        assert_close!(actual_re, expected_re, rtol = 0.0, atol = bound);
        assert_close!(actual_im, expected_im, rtol = 0.0, atol = bound);
    }

    #[test]
    fn test_fir_i16_saturates() {
        const FRACTIONAL_BITS: u32 = 15;

        let coefficients = [Complex::new(i16::MAX, 0), Complex::new(i16::MAX, 0)];

        let mut filter = FirFilter::<Complex<i16>>::new(&coefficients, FRACTIONAL_BITS);

        let input = Complex::new(i16::MAX, 0);

        let _ = filter.process_sample(input);
        let output = filter.process_sample(input);

        assert_eq!(output.re, i16::MAX);
        assert_eq!(output.im, 0);
    }

    #[test]
    fn test_fir_i8_saturates() {
        const FRACTIONAL_BITS: u32 = 7;

        let coefficients = [Complex::new(i8::MAX, 0), Complex::new(i8::MAX, 0)];

        let mut filter = FirFilter::<Complex<i8>>::new(&coefficients, FRACTIONAL_BITS);

        let input = Complex::new(i8::MAX, 0);

        let _ = filter.process_sample(input);
        let output = filter.process_sample(input);

        assert_eq!(output.re, i8::MAX);
        assert_eq!(output.im, 0);
    }

    #[test]
    fn test_fir_reset_clears_state() {
        let coefficients = [Complex::new(0.5_f64, 0.0), Complex::new(0.5_f64, 0.0)];

        let mut filter = FirFilter::<Complex<f64>>::new(&coefficients);

        let _ = filter.process_sample(Complex::new(1.0, 0.0));
        filter.reset();

        let output = filter.process_sample(Complex::new(0.0, 0.0));

        assert_eq!(output, Complex::new(0.0, 0.0));
    }
}
