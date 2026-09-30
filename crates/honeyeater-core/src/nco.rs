use std::marker::PhantomData;

use num_complex::Complex;

use crate::Sample;

// A full DDS phase turn contains 2^32 phase words.
const PHASE_MODULUS: f64 = 4_294_967_296.0;

/// A stateful numerically controlled oscillator / direct digital synthesiser.
///
/// Frequency is expressed in cycles per sample. For example:
///
/// - `0.25` = Fs / 4
/// - `-0.25` = -Fs / 4
/// - `0.0` = DC
///
/// Frequencies are periodic modulo one cycle per sample, so values outside
/// `[-0.5, 0.5)` alias in the normal discrete-time sense.
///
/// The oscillator uses a 32-bit phase accumulator shared by all output sample
/// types. The first generated sample is at the configured initial phase, then
/// the phase accumulator advances by one frequency word.
///
/// `T` is the complex output sample type.
pub struct Nco<T: Sample> {
    phase: u32,
    phase_increment: u32,
    _sample: PhantomData<T>,
}

impl<T: Sample> Nco<T> {
    /// Creates an NCO starting at zero phase.
    ///
    /// `frequency` is specified in cycles per sample.
    ///
    /// # Panics
    ///
    /// Panics if `frequency` is not finite.
    #[must_use]
    pub fn new(frequency: f64) -> Self {
        Self::with_phase(frequency, 0.0)
    }

    /// Creates an NCO with the supplied initial phase.
    ///
    /// Both `frequency` and `phase` are specified in cycles. Phase is wrapped
    /// into the interval `[0, 1)`.
    ///
    /// # Panics
    ///
    /// Panics if `frequency` or `phase` is not finite.
    #[must_use]
    pub fn with_phase(frequency: f64, phase: f64) -> Self {
        assert!(frequency.is_finite(), "NCO frequency must be finite");
        assert!(phase.is_finite(), "NCO phase must be finite");

        Self {
            phase: cycles_to_phase_word(phase),
            phase_increment: cycles_to_phase_word(frequency),
            _sample: PhantomData,
        }
    }

    /// Changes the oscillator frequency while preserving phase continuity.
    ///
    /// `frequency` is specified in cycles per sample.
    ///
    /// # Panics
    ///
    /// Panics if `frequency` is not finite.
    pub fn set_frequency(&mut self, frequency: f64) {
        assert!(frequency.is_finite(), "NCO frequency must be finite");
        self.phase_increment = cycles_to_phase_word(frequency);
    }

    /// Sets the oscillator phase in cycles.
    ///
    /// Phase is wrapped into the interval `[0, 1)`.
    ///
    /// # Panics
    ///
    /// Panics if `phase` is not finite.
    pub fn set_phase(&mut self, phase: f64) {
        assert!(phase.is_finite(), "NCO phase must be finite");
        self.phase = cycles_to_phase_word(phase);
    }

    /// Returns the current oscillator phase in cycles in `[0, 1)`.
    #[must_use]
    pub fn phase(&self) -> f64 {
        f64::from(self.phase) / PHASE_MODULUS
    }

    fn next_unit_phasor(&mut self) -> (f64, f64) {
        let phase = f64::from(self.phase) / PHASE_MODULUS;
        let angle = std::f64::consts::TAU * phase;

        let (sin, cos) = angle.sin_cos();

        self.phase = self.phase.wrapping_add(self.phase_increment);

        (cos, sin)
    }
}

/// Convert normalised phase/frequency in cycles into a 32-bit DDS phase word.
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn cycles_to_phase_word(cycles: f64) -> u32 {
    let wrapped = cycles.rem_euclid(1.0);
    let word = (wrapped * PHASE_MODULUS).round() as u64;

    // The u64 -> u32 truncation is intentional. A value which rounds to
    // exactly 2^32 represents the same phase as zero.
    word as u32
}

#[allow(clippy::cast_possible_truncation)]
fn complex_f32(re: f64, im: f64) -> Complex<f32> {
    Complex::new(re as f32, im as f32)
}

/// Quantise a unit-amplitude value into signed Q1.15.
///
/// Positive full scale saturates at +32767 while -1.0 is represented exactly
/// by -32768.
#[allow(clippy::cast_possible_truncation)]
fn quantize_q15(value: f64) -> i16 {
    let scaled = (value * 32_768.0)
        .round()
        .clamp(f64::from(i16::MIN), f64::from(i16::MAX));

    scaled as i16
}

/// Quantise a unit-amplitude value into signed Q1.7.
///
/// Positive full scale saturates at +127 while -1.0 is represented exactly
/// by -128.
#[allow(clippy::cast_possible_truncation)]
fn quantize_q7(value: f64) -> i8 {
    let scaled = (value * 128.0)
        .round()
        .clamp(f64::from(i8::MIN), f64::from(i8::MAX));

    scaled as i8
}

impl Nco<Complex<f64>> {
    /// Generates the next unit-amplitude `Complex<f64>` oscillator sample.
    pub fn next_sample(&mut self) -> Complex<f64> {
        let (re, im) = self.next_unit_phasor();
        Complex::new(re, im)
    }

    /// Fills `output` with consecutive oscillator samples.
    pub fn fill(&mut self, output: &mut [Complex<f64>]) {
        for sample in output {
            *sample = self.next_sample();
        }
    }
}

impl Nco<Complex<f32>> {
    /// Generates the next unit-amplitude `Complex<f32>` oscillator sample.
    pub fn next_sample(&mut self) -> Complex<f32> {
        let (re, im) = self.next_unit_phasor();
        complex_f32(re, im)
    }

    /// Fills `output` with consecutive oscillator samples.
    pub fn fill(&mut self, output: &mut [Complex<f32>]) {
        for sample in output {
            *sample = self.next_sample();
        }
    }
}

impl Nco<Complex<i16>> {
    /// Generates the next Q1.15 `Complex<i16>` oscillator sample.
    pub fn next_sample(&mut self) -> Complex<i16> {
        let (re, im) = self.next_unit_phasor();

        Complex::new(quantize_q15(re), quantize_q15(im))
    }

    /// Fills `output` with consecutive Q1.15 oscillator samples.
    pub fn fill(&mut self, output: &mut [Complex<i16>]) {
        for sample in output {
            *sample = self.next_sample();
        }
    }
}

impl Nco<Complex<i8>> {
    /// Generates the next Q1.7 `Complex<i8>` oscillator sample.
    pub fn next_sample(&mut self) -> Complex<i8> {
        let (re, im) = self.next_unit_phasor();

        Complex::new(quantize_q7(re), quantize_q7(im))
    }

    /// Fills `output` with consecutive Q1.7 oscillator samples.
    pub fn fill(&mut self, output: &mut [Complex<i8>]) {
        for sample in output {
            *sample = self.next_sample();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use honeyeater_test::{assert_bit_exact, assert_close, assert_spectral_mask, npy};
    use rustfft::FftPlanner;
    use std::path::PathBuf;

    const ORACLE_LENGTH: usize = 64;
    const ORACLE_FREQUENCY: f64 = 5.0 / 64.0;
    const SFDR_FREQUENCY: f64 = 37.0 / 4096.0;

    fn vector_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        path.push("tests");
        path.push("vectors");
        path.push("nco");
        path.push(filename);
        path
    }

    fn assert_complex_f64_close(
        actual: &[Complex<f64>],
        expected: &[Complex<f64>],
        rtol: f64,
        atol: f64,
    ) {
        let actual_re: Vec<f64> = actual.iter().map(|sample| sample.re).collect();
        let actual_im: Vec<f64> = actual.iter().map(|sample| sample.im).collect();

        let expected_re: Vec<f64> = expected.iter().map(|sample| sample.re).collect();
        let expected_im: Vec<f64> = expected.iter().map(|sample| sample.im).collect();

        assert_close!(actual_re, expected_re, rtol = rtol, atol = atol);
        assert_close!(actual_im, expected_im, rtol = rtol, atol = atol);
    }

    /// Checks SFDR by normalising every FFT bin against the carrier and then
    /// applying Honeyeater's existing spectral-mask assertion.
    ///
    /// The input tone is deliberately coherent with the FFT length, so
    /// spectral leakage does not need a window and any other bin is a spur.
    fn assert_sfdr(mut samples: Vec<Complex<f64>>, carrier_bin: usize, minimum_sfdr_db: f64) {
        assert!(
            carrier_bin < samples.len(),
            "carrier bin must lie inside FFT"
        );

        let mut planner = FftPlanner::<f64>::new();
        let fft = planner.plan_fft_forward(samples.len());
        fft.process(&mut samples);

        let carrier_magnitude = samples[carrier_bin].norm();

        assert!(
            carrier_magnitude > 0.0,
            "NCO carrier magnitude must be greater than zero"
        );

        let bins_db: Vec<f64> = samples
            .iter()
            .map(|bin| 20.0 * (bin.norm() / carrier_magnitude).log10())
            .collect();

        let lower = vec![f64::NEG_INFINITY; bins_db.len()];
        let mut upper = vec![-minimum_sfdr_db; bins_db.len()];

        // The carrier itself is the 0 dBc reference and is not a spur.
        upper[carrier_bin] = f64::INFINITY;

        assert_spectral_mask!(bins_db, lower = lower, upper = upper);
    }

    #[test]
    fn test_nco_f64_matches_numpy_oracle() {
        let expected = npy::load_complex_f64(&vector_path("nco_f64_64.npy"))
            .expect("failed to load f64 NCO reference vector");

        let mut nco = Nco::<Complex<f64>>::new(ORACLE_FREQUENCY);
        let mut actual = vec![Complex::default(); ORACLE_LENGTH];

        // Split the output deliberately to verify that state carries correctly
        // between successive buffer calls.
        let (first, second) = actual.split_at_mut(17);
        nco.fill(first);
        nco.fill(second);

        assert_complex_f64_close(&actual, &expected, 1e-12, 1e-15);
    }

    #[test]
    fn test_nco_f32_matches_numpy_oracle() {
        let expected = npy::load_complex_f32(&vector_path("nco_f32_64.npy"))
            .expect("failed to load f32 NCO reference vector");

        let mut nco = Nco::<Complex<f32>>::new(ORACLE_FREQUENCY);
        let mut actual = vec![Complex::default(); ORACLE_LENGTH];
        nco.fill(&mut actual);

        let actual_f64: Vec<Complex<f64>> = actual
            .iter()
            .map(|sample| Complex::new(f64::from(sample.re), f64::from(sample.im)))
            .collect();

        let expected_f64: Vec<Complex<f64>> = expected
            .iter()
            .map(|sample| Complex::new(f64::from(sample.re), f64::from(sample.im)))
            .collect();

        assert_complex_f64_close(&actual_f64, &expected_f64, 1e-6, 1e-7);
    }

    #[test]
    fn test_nco_i16_matches_numpy_oracle_bit_exact() {
        let expected = npy::load_complex_i16(&vector_path("nco_i16_64.npy"))
            .expect("failed to load i16 NCO reference vector");

        let mut nco = Nco::<Complex<i16>>::new(ORACLE_FREQUENCY);
        let mut actual = vec![Complex::default(); ORACLE_LENGTH];
        nco.fill(&mut actual);

        assert_bit_exact!(actual, expected);
    }

    #[test]
    fn test_nco_i8_matches_numpy_oracle_bit_exact() {
        let expected = npy::load_complex_i8(&vector_path("nco_i8_64.npy"))
            .expect("failed to load i8 NCO reference vector");

        let mut nco = Nco::<Complex<i8>>::new(ORACLE_FREQUENCY);
        let mut actual = vec![Complex::default(); ORACLE_LENGTH];
        nco.fill(&mut actual);

        assert_bit_exact!(actual, expected);
    }

    #[test]
    fn test_nco_f64_sfdr() {
        const LENGTH: usize = 4096;
        const CARRIER_BIN: usize = 37;

        let mut nco = Nco::<Complex<f64>>::new(SFDR_FREQUENCY);

        let mut samples = vec![Complex::default(); LENGTH];
        nco.fill(&mut samples);

        assert_sfdr(samples, CARRIER_BIN, 120.0);
    }

    #[test]
    fn test_nco_f32_sfdr() {
        const LENGTH: usize = 4096;
        const CARRIER_BIN: usize = 37;

        let mut nco = Nco::<Complex<f32>>::new(SFDR_FREQUENCY);

        let mut native = vec![Complex::default(); LENGTH];
        nco.fill(&mut native);

        let samples: Vec<Complex<f64>> = native
            .into_iter()
            .map(|sample| Complex::new(f64::from(sample.re), f64::from(sample.im)))
            .collect();

        assert_sfdr(samples, CARRIER_BIN, 100.0);
    }

    #[test]
    fn test_nco_i16_sfdr() {
        const LENGTH: usize = 4096;
        const CARRIER_BIN: usize = 37;

        let mut nco = Nco::<Complex<i16>>::new(SFDR_FREQUENCY);

        let mut native = vec![Complex::default(); LENGTH];
        nco.fill(&mut native);

        let samples: Vec<Complex<f64>> = native
            .into_iter()
            .map(|sample| Complex::new(f64::from(sample.re), f64::from(sample.im)))
            .collect();

        assert_sfdr(samples, CARRIER_BIN, 90.0);
    }

    #[test]
    fn test_nco_i8_sfdr() {
        const LENGTH: usize = 4096;
        const CARRIER_BIN: usize = 37;

        let mut nco = Nco::<Complex<i8>>::new(SFDR_FREQUENCY);

        let mut native = vec![Complex::default(); LENGTH];
        nco.fill(&mut native);

        let samples: Vec<Complex<f64>> = native
            .into_iter()
            .map(|sample| Complex::new(f64::from(sample.re), f64::from(sample.im)))
            .collect();

        assert_sfdr(samples, CARRIER_BIN, 50.0);
    }

    #[test]
    fn test_set_frequency_preserves_phase() {
        let mut nco = Nco::<Complex<f64>>::new(1.0 / 16.0);

        let _ = nco.next_sample();
        let phase_before = nco.phase();

        nco.set_frequency(1.0 / 8.0);

        assert_eq!(nco.phase().to_bits(), phase_before.to_bits());
    }
}
