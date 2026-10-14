use num_complex::Complex;

use crate::q_format::{QFormat, RTL_SDR_U8};

fn assert_equal_lengths(input_len: usize, output_len: usize) {
    assert_eq!(
        input_len, output_len,
        "SDR boundary input and output lengths must match"
    );
}

fn assert_i8_q_format(format: QFormat) {
    assert!(
        format.fractional_bits() <= 7,
        "Complex<i8> conversion requires <= 7 fractional bits"
    );
}

#[allow(clippy::cast_possible_truncation)]
fn quantize_f32_i16(value: f32, format: QFormat) -> i16 {
    let scale = format.scale_f32();

    let minimum = -scale;
    let maximum = scale - 1.0;

    (value * scale).round().clamp(minimum, maximum) as i16
}

#[allow(clippy::cast_possible_truncation)]
fn quantize_f64_i16(value: f64, format: QFormat) -> i16 {
    let scale = format.scale_f64();

    let minimum = -scale;
    let maximum = scale - 1.0;

    (value * scale).round().clamp(minimum, maximum) as i16
}

#[allow(clippy::cast_possible_truncation)]
fn quantize_f32_i8(value: f32, format: QFormat) -> i8 {
    assert_i8_q_format(format);

    let scale = format.scale_f32();

    let minimum = -scale;
    let maximum = scale - 1.0;

    (value * scale).round().clamp(minimum, maximum) as i8
}

#[allow(clippy::cast_possible_truncation)]
fn quantize_f64_i8(value: f64, format: QFormat) -> i8 {
    assert_i8_q_format(format);

    let scale = format.scale_f64();

    let minimum = -scale;
    let maximum = scale - 1.0;

    (value * scale).round().clamp(minimum, maximum) as i8
}

/// Converts signed 16-bit complex samples to normalised `Complex<f32>`.
///
/// Each integer component is divided by the scale represented by `format`.
///
/// # Panics
///
/// Panics if `input` and `output` have different lengths.
pub fn complex_i16_to_f32(input: &[Complex<i16>], output: &mut [Complex<f32>], format: QFormat) {
    assert_equal_lengths(input.len(), output.len());

    let scale = format.scale_f32();

    for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
        *output_sample = Complex::new(
            f32::from(input_sample.re) / scale,
            f32::from(input_sample.im) / scale,
        );
    }
}

/// Converts signed 16-bit complex samples to normalised `Complex<f64>`.
///
/// # Panics
///
/// Panics if `input` and `output` have different lengths.
pub fn complex_i16_to_f64(input: &[Complex<i16>], output: &mut [Complex<f64>], format: QFormat) {
    assert_equal_lengths(input.len(), output.len());

    let scale = format.scale_f64();

    for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
        *output_sample = Complex::new(
            f64::from(input_sample.re) / scale,
            f64::from(input_sample.im) / scale,
        );
    }
}

/// Converts signed 8-bit complex samples to normalised `Complex<f32>`.
///
/// # Panics
///
/// Panics if `input` and `output` have different lengths or if `format` uses
/// more than seven fractional bits.
pub fn complex_i8_to_f32(input: &[Complex<i8>], output: &mut [Complex<f32>], format: QFormat) {
    assert_equal_lengths(input.len(), output.len());
    assert_i8_q_format(format);

    let scale = format.scale_f32();

    for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
        *output_sample = Complex::new(
            f32::from(input_sample.re) / scale,
            f32::from(input_sample.im) / scale,
        );
    }
}

/// Converts signed 8-bit complex samples to normalised `Complex<f64>`.
///
/// # Panics
///
/// Panics if `input` and `output` have different lengths or if `format` uses
/// more than seven fractional bits.
pub fn complex_i8_to_f64(input: &[Complex<i8>], output: &mut [Complex<f64>], format: QFormat) {
    assert_equal_lengths(input.len(), output.len());
    assert_i8_q_format(format);

    let scale = format.scale_f64();

    for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
        *output_sample = Complex::new(
            f64::from(input_sample.re) / scale,
            f64::from(input_sample.im) / scale,
        );
    }
}

/// Quantises `Complex<f32>` samples into a signed 16-bit Q-format.
///
/// Values outside the representable normalised Q-format range are saturated.
///
/// For Q1.11, for example, the output range is `[-2048, 2047]` rather than
/// the entire `i16` range. This prevents generation of invalid BladeRF
/// `SC16_Q11` values in the otherwise-unused container headroom.
///
/// # Panics
///
/// Panics if `input` and `output` have different lengths.
pub fn complex_f32_to_i16(input: &[Complex<f32>], output: &mut [Complex<i16>], format: QFormat) {
    assert_equal_lengths(input.len(), output.len());

    for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
        *output_sample = Complex::new(
            quantize_f32_i16(input_sample.re, format),
            quantize_f32_i16(input_sample.im, format),
        );
    }
}

/// Quantises `Complex<f64>` samples into a signed 16-bit Q-format.
///
/// Values outside the representable normalised Q-format range are saturated.
///
/// # Panics
///
/// Panics if `input` and `output` have different lengths.
pub fn complex_f64_to_i16(input: &[Complex<f64>], output: &mut [Complex<i16>], format: QFormat) {
    assert_equal_lengths(input.len(), output.len());

    for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
        *output_sample = Complex::new(
            quantize_f64_i16(input_sample.re, format),
            quantize_f64_i16(input_sample.im, format),
        );
    }
}

/// Quantises `Complex<f32>` samples into a signed 8-bit Q-format.
///
/// Values outside the representable Q-format range are saturated.
///
/// # Panics
///
/// Panics if `input` and `output` have different lengths or if `format` uses
/// more than seven fractional bits.
pub fn complex_f32_to_i8(input: &[Complex<f32>], output: &mut [Complex<i8>], format: QFormat) {
    assert_equal_lengths(input.len(), output.len());
    assert_i8_q_format(format);

    for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
        *output_sample = Complex::new(
            quantize_f32_i8(input_sample.re, format),
            quantize_f32_i8(input_sample.im, format),
        );
    }
}

/// Quantises `Complex<f64>` samples into a signed 8-bit Q-format.
///
/// Values outside the representable Q-format range are saturated.
///
/// # Panics
///
/// Panics if `input` and `output` have different lengths or if `format` uses
/// more than seven fractional bits.
pub fn complex_f64_to_i8(input: &[Complex<f64>], output: &mut [Complex<i8>], format: QFormat) {
    assert_equal_lengths(input.len(), output.len());
    assert_i8_q_format(format);

    for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
        *output_sample = Complex::new(
            quantize_f64_i8(input_sample.re, format),
            quantize_f64_i8(input_sample.im, format),
        );
    }
}

/// Debiases raw RTL-SDR unsigned IQ samples into `Complex<i8>`.
///
/// An integer type cannot represent the exact 127.5 midpoint. The signed path
/// therefore maps the raw byte range bijectively by subtracting 128:
///
/// - `0   -> -128`
/// - `127 -> -1`
/// - `128 -> 0`
/// - `255 -> 127`
///
/// This preserves all 256 possible raw values and is exactly reversible.
///
/// # Panics
///
/// Panics if `input` and `output` have different lengths.
pub fn rtl_sdr_u8_to_i8(input: &[Complex<u8>], output: &mut [Complex<i8>]) {
    assert_equal_lengths(input.len(), output.len());

    for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
        let re = i16::from(input_sample.re) - 128;
        let im = i16::from(input_sample.im) - 128;

        *output_sample = Complex::new(
            i8::try_from(re).expect("RTL-SDR debiased value must fit in i8"),
            i8::try_from(im).expect("RTL-SDR debiased value must fit in i8"),
        );
    }
}

/// Re-biases signed RTL-SDR samples into their original unsigned byte format.
///
/// This is the exact inverse of [`rtl_sdr_u8_to_i8`].
///
/// # Panics
///
/// Panics if `input` and `output` have different lengths.
pub fn rtl_sdr_i8_to_u8(input: &[Complex<i8>], output: &mut [Complex<u8>]) {
    assert_equal_lengths(input.len(), output.len());

    for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
        let re = i16::from(input_sample.re) + 128;
        let im = i16::from(input_sample.im) + 128;

        *output_sample = Complex::new(
            u8::try_from(re).expect("RTL-SDR re-biased value must fit in u8"),
            u8::try_from(im).expect("RTL-SDR re-biased value must fit in u8"),
        );
    }
}

/// Converts raw RTL-SDR unsigned IQ samples directly to normalised
/// `Complex<f32>`.
///
/// Unlike the integer path, floating point can preserve the true 127.5 DC
/// midpoint exactly:
///
/// `output = (input - 127.5) / 128`
///
/// Consequently raw values 127 and 128 lie symmetrically around zero.
///
/// # Panics
///
/// Panics if `input` and `output` have different lengths.
pub fn rtl_sdr_u8_to_f32(input: &[Complex<u8>], output: &mut [Complex<f32>]) {
    assert_equal_lengths(input.len(), output.len());

    let midpoint = RTL_SDR_U8.midpoint();
    let scale = RTL_SDR_U8.scale();

    for (&input_sample, output_sample) in input.iter().zip(output.iter_mut()) {
        *output_sample = Complex::new(
            (f32::from(input_sample.re) - midpoint) / scale,
            (f32::from(input_sample.im) - midpoint) / scale,
        );
    }
}

/// Separates interleaved complex samples into independent I and Q arrays.
///
/// No allocation is performed.
///
/// # Panics
///
/// Panics unless `i_output` and `q_output` both have the same length as
/// `input`.
pub fn deinterleave_complex<T: Copy>(input: &[Complex<T>], i_output: &mut [T], q_output: &mut [T]) {
    assert_equal_lengths(input.len(), i_output.len());
    assert_equal_lengths(input.len(), q_output.len());

    for ((sample, i_value), q_value) in input
        .iter()
        .zip(i_output.iter_mut())
        .zip(q_output.iter_mut())
    {
        *i_value = sample.re;
        *q_value = sample.im;
    }
}

/// Combines separate I and Q arrays into interleaved complex samples.
///
/// This is the inverse of [`deinterleave_complex`].
///
/// # Panics
///
/// Panics unless `i_input`, `q_input`, and `output` all have equal lengths.
pub fn interleave_complex<T: Copy>(i_input: &[T], q_input: &[T], output: &mut [Complex<T>]) {
    assert_equal_lengths(i_input.len(), q_input.len());
    assert_equal_lengths(i_input.len(), output.len());

    for ((&i_value, &q_value), output_sample) in
        i_input.iter().zip(q_input.iter()).zip(output.iter_mut())
    {
        *output_sample = Complex::new(i_value, q_value);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::q_format;
    use honeyeater_test::assert_bit_exact;

    #[test]
    fn i16_q15_f32_round_trip_is_exact() {
        let input = [
            Complex::new(-32_768, 32_767),
            Complex::new(-16_384, 16_384),
            Complex::new(-1, 1),
            Complex::new(0, 0),
        ];

        let mut float = [Complex::new(0.0_f32, 0.0); 4];
        let mut actual = [Complex::new(0_i16, 0); 4];

        complex_i16_to_f32(&input, &mut float, q_format::USRP_SC16);

        complex_f32_to_i16(&float, &mut actual, q_format::USRP_SC16);

        assert_bit_exact!(actual, input);
    }

    #[test]
    fn i16_q11_f64_round_trip_is_exact() {
        let input = [
            Complex::new(-2_048, 2_047),
            Complex::new(-1_024, 1_024),
            Complex::new(-1, 1),
            Complex::new(0, 0),
        ];

        let mut float = [Complex::new(0.0_f64, 0.0); 4];
        let mut actual = [Complex::new(0_i16, 0); 4];

        complex_i16_to_f64(&input, &mut float, q_format::BLADERF_SC16_Q11);

        complex_f64_to_i16(&float, &mut actual, q_format::BLADERF_SC16_Q11);

        assert_bit_exact!(actual, input);
    }

    #[test]
    fn i8_q7_f32_round_trip_is_exact() {
        let input = [
            Complex::new(-128, 127),
            Complex::new(-64, 64),
            Complex::new(-1, 1),
            Complex::new(0, 0),
        ];

        let mut float = [Complex::new(0.0_f32, 0.0); 4];
        let mut actual = [Complex::new(0_i8, 0); 4];

        complex_i8_to_f32(&input, &mut float, q_format::HACKRF_SC8);

        complex_f32_to_i8(&float, &mut actual, q_format::HACKRF_SC8);

        assert_bit_exact!(actual, input);
    }

    #[test]
    #[allow(clippy::float_cmp)] // Exact equality is intentional for these binary-exact values.
    fn q15_normalised_range_is_correct() {
        let input = [Complex::new(i16::MIN, i16::MAX)];

        let mut output = [Complex::new(0.0_f32, 0.0)];

        complex_i16_to_f32(&input, &mut output, q_format::Q1_15);

        assert_eq!(output[0].re, -1.0);
        assert_eq!(output[0].im, f32::from(i16::MAX) / 32_768.0);

        assert!(output[0].im < 1.0);
    }

    #[test]
    fn q11_output_saturates_to_q11_range() {
        let input = [Complex::new(-2.0_f32, 2.0_f32)];

        let mut output = [Complex::new(0_i16, 0)];

        complex_f32_to_i16(&input, &mut output, q_format::BLADERF_SC16_Q11);

        assert_eq!(output[0], Complex::new(-2_048, 2_047));
    }

    #[test]
    fn rtl_sdr_integer_bias_mapping_is_correct() {
        let input = [Complex::new(0_u8, 127_u8), Complex::new(128_u8, 255_u8)];

        let mut output = [Complex::new(0_i8, 0_i8); 2];

        rtl_sdr_u8_to_i8(&input, &mut output);

        let expected = [Complex::new(-128_i8, -1_i8), Complex::new(0_i8, 127_i8)];

        assert_bit_exact!(output, expected);
    }

    #[test]
    fn rtl_sdr_integer_path_round_trips_all_byte_values() {
        let input: Vec<Complex<u8>> = (u8::MIN..=u8::MAX)
            .map(|value| Complex::new(value, u8::MAX - value))
            .collect();

        let mut signed = vec![Complex::new(0_i8, 0_i8); input.len()];
        let mut actual = vec![Complex::new(0_u8, 0_u8); input.len()];

        rtl_sdr_u8_to_i8(&input, &mut signed);
        rtl_sdr_i8_to_u8(&signed, &mut actual);

        assert_bit_exact!(actual, input);
    }

    #[test]
    #[allow(clippy::float_cmp)] // Exact equality is intentional for these binary-exact values.
    fn rtl_sdr_float_path_preserves_true_midpoint() {
        let input = [Complex::new(127_u8, 128_u8), Complex::new(0_u8, 255_u8)];

        let mut output = [Complex::new(0.0_f32, 0.0_f32); 2];

        rtl_sdr_u8_to_f32(&input, &mut output);

        assert_eq!(output[0].re, -0.5 / 128.0);
        assert_eq!(output[0].im, 0.5 / 128.0);

        assert_eq!(output[1].re, -127.5 / 128.0);
        assert_eq!(output[1].im, 127.5 / 128.0);

        assert_eq!(output[0].re, -output[0].im);
        assert_eq!(output[1].re, -output[1].im);
    }

    #[test]
    fn deinterleave_and_interleave_round_trip_is_exact() {
        let input = [
            Complex::new(1_i16, -1_i16),
            Complex::new(2_i16, -2_i16),
            Complex::new(3_i16, -3_i16),
            Complex::new(4_i16, -4_i16),
        ];

        let mut i = [0_i16; 4];
        let mut q = [0_i16; 4];

        deinterleave_complex(&input, &mut i, &mut q);

        assert_bit_exact!(i, [1, 2, 3, 4]);
        assert_bit_exact!(q, [-1, -2, -3, -4]);

        let mut actual = [Complex::new(0_i16, 0_i16); 4];

        interleave_complex(&i, &q, &mut actual);

        assert_bit_exact!(actual, input);
    }
}
