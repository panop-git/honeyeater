//! Public Phase 1 contracts checked against committed NumPy/SciPy references.

use honeyeater_core::{
    Biquad, Complex, FftWrapper, RustFftBackend, complex_magnitudes, complex_powers, kaiser_window,
};
use honeyeater_test::{assert_bit_exact, assert_close, assert_snr_db, npy};
use std::path::PathBuf;

fn fixture(name: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("tests/vectors/phase1_audit")
        .join(name)
}

#[test]
fn public_fft_matches_scipy_for_odd_and_even_lengths() {
    for size in [2, 3, 7, 15, 65] {
        let expected_input =
            npy::load_complex_f64(&fixture(&format!("fft_input_{size}.npy"))).unwrap();
        let expected_output =
            npy::load_complex_f64(&fixture(&format!("fft_output_{size}.npy"))).unwrap();
        let expected_frequency =
            npy::load_f64(&fixture(&format!("fft_frequency_{size}.npy"))).unwrap();
        let backend = RustFftBackend::<f64>::new(size);
        let mut buffer = expected_input.clone();
        backend.fft(&mut buffer);
        let flat = |samples: &[Complex<f64>]| {
            samples
                .iter()
                .flat_map(|v| [v.re, v.im])
                .collect::<Vec<_>>()
        };
        assert_snr_db!(flat(&buffer), flat(&expected_output), min_db = 120.0);
        backend.ifft(&mut buffer);
        assert_snr_db!(flat(&buffer), flat(&expected_input), min_db = 120.0);
        let frequency: Vec<_> = (0..size).map(|bin| backend.bin_frequency(bin)).collect();
        assert_close!(frequency, expected_frequency, rtol = 1e-12, atol = 1e-15);
    }
}

#[test]
#[should_panic(expected = "at least two")]
fn fft_rejects_length_one() {
    let _ = RustFftBackend::<f64>::new(1);
}

#[test]
#[should_panic(expected = "configured FFT size")]
fn fft_rejects_mismatched_buffers() {
    let backend = RustFftBackend::<f64>::new(8);
    backend.fft(&mut [Complex::default(); 16]);
}

#[test]
fn kaiser_matches_scipy_across_beta_range() {
    for beta in [0, 14, 30, 50, 100, 700] {
        let expected = npy::load_f64(&fixture(&format!("kaiser_{beta}.npy"))).unwrap();
        let actual: Vec<f64> = (0..65)
            .map(|index| kaiser_window(index, 65, f64::from(beta)))
            .collect();
        assert_close!(actual, expected, rtol = 1e-12, atol = 1e-15);
    }
}

#[test]
fn biquad_coefficients_and_streaming_match_scipy() {
    for (case, (frequency, q)) in [(0.01, 0.5), (0.125, 1.0), (0.45, 10.0)]
        .into_iter()
        .enumerate()
    {
        let mut filter = Biquad::<f64>::lowpass_normalized(frequency, q);
        let (b, a) = filter.coefficients();
        let coefficients: Vec<_> = b.into_iter().chain(a).collect();
        let expected = npy::load_f64(&fixture(&format!("biquad_coefficients_{case}.npy"))).unwrap();
        assert_close!(coefficients, expected, rtol = 1e-12, atol = 1e-15);
        let mut impulse = [0.0_f64; 256];
        impulse[0] = 1.0;
        let mut output = [0.0; 256];
        for (input, output) in impulse.chunks(13).zip(output.chunks_mut(13)) {
            filter.process(input, output);
        }
        let expected = npy::load_f64(&fixture(&format!("biquad_output_{case}.npy"))).unwrap();
        assert_snr_db!(output, expected, min_db = 80.0);
        filter.reset();
        assert_close!(
            filter.process_owned(&impulse),
            expected,
            rtol = 1e-12,
            atol = 1e-15
        );
    }
}

#[test]
fn integer_magnitude_and_power_match_numpy_bit_exactly() {
    let input = npy::load_complex_i8(&fixture("magnitude_input_i8.npy")).unwrap();
    let mut magnitude = vec![0_u32; input.len()];
    let mut power = vec![0_u32; input.len()];
    complex_magnitudes(&input, &mut magnitude);
    complex_powers(&input, &mut power);
    assert_bit_exact!(
        magnitude,
        npy::load_u32(&fixture("magnitude_i8.npy")).unwrap()
    );
    assert_bit_exact!(power, npy::load_u32(&fixture("power_i8.npy")).unwrap());
    let input = npy::load_complex_i16(&fixture("magnitude_input_i16.npy")).unwrap();
    let mut magnitude = vec![0_u32; input.len()];
    let mut power = vec![0_u32; input.len()];
    complex_magnitudes(&input, &mut magnitude);
    complex_powers(&input, &mut power);
    assert_bit_exact!(
        magnitude,
        npy::load_u32(&fixture("magnitude_i16.npy")).unwrap()
    );
    assert_bit_exact!(power, npy::load_u32(&fixture("power_i16.npy")).unwrap());
}

#[test]
fn float_magnitude_and_power_match_numpy_across_dynamic_range() {
    let input = npy::load_complex_f64(&fixture("magnitude_input_f64.npy")).unwrap();
    let mut magnitude = vec![0.0_f64; input.len()];
    let mut power = vec![0.0_f64; input.len()];
    complex_magnitudes(&input, &mut magnitude);
    complex_powers(&input, &mut power);
    assert_close!(
        magnitude,
        npy::load_f64(&fixture("magnitude_f64.npy")).unwrap(),
        rtol = 1e-12,
        atol = 0.0
    );
    assert_close!(
        power,
        npy::load_f64(&fixture("power_f64.npy")).unwrap(),
        rtol = 1e-12,
        atol = 0.0
    );
}
