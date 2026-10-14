use pm_remez::{BandSetting, constant, pm_parameters, pm_remez as parks_mcclellan};

use crate::{
    blackmanharris_window, hamming_window, hann_window, kaiser_window,
};

/// Window functions available to the window-method FIR designers.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum FirWindow {
    /// Rectangular window.
    Rectangular,

    /// Symmetric Hann window.
    Hann,

    /// Symmetric Hamming window.
    Hamming,

    /// Symmetric Blackman-Harris window.
    BlackmanHarris,

    /// Symmetric Kaiser window with the supplied beta parameter.
    Kaiser(f64),
}

impl FirWindow {
    fn value(self, n: usize, length: usize) -> f64 {
        match self {
            Self::Rectangular => 1.0,
            Self::Hann => hann_window(n, length),
            Self::Hamming => hamming_window(n, length),
            Self::BlackmanHarris => blackmanharris_window(n, length),
            Self::Kaiser(beta) => kaiser_window(n, length, beta),
        }
    }
}

fn usize_to_f64(value: usize) -> f64 {
    let value = u32::try_from(value).expect("FIR tap count or index exceeds u32::MAX");
    f64::from(value)
}

fn validate_common(num_taps: usize, sample_rate: f64) {
    assert!(num_taps > 0, "FIR design requires at least one tap");

    assert!(
        sample_rate.is_finite() && sample_rate > 0.0,
        "FIR sample rate must be finite and greater than zero"
    );
}

fn validate_cutoff(cutoff_hz: f64, sample_rate: f64) {
    let nyquist = sample_rate / 2.0;

    assert!(
        cutoff_hz.is_finite() && cutoff_hz > 0.0 && cutoff_hz < nyquist,
        "FIR cutoff must be finite and strictly between zero and Nyquist"
    );
}

fn validate_band(
    lower_cutoff_hz: f64,
    upper_cutoff_hz: f64,
    sample_rate: f64,
) {
    validate_cutoff(lower_cutoff_hz, sample_rate);
    validate_cutoff(upper_cutoff_hz, sample_rate);

    assert!(
        lower_cutoff_hz < upper_cutoff_hz,
        "FIR lower cutoff must be below upper cutoff"
    );
}

/// Normalised sinc function.
///
/// `sinc(x) = sin(pi*x) / (pi*x)`.
fn sinc(x: f64) -> f64 {
    if x.abs() <= f64::EPSILON {
        1.0
    } else {
        let argument = std::f64::consts::PI * x;
        argument.sin() / argument
    }
}

/// Ideal low-pass impulse response evaluated at a displacement from the
/// centre tap.
///
/// `cutoff_hz / sample_rate` is expressed in cycles per sample.
fn ideal_lowpass(offset: f64, cutoff_hz: f64, sample_rate: f64) -> f64 {
    let normalized_cutoff = cutoff_hz / sample_rate;

    2.0 * normalized_cutoff
        * sinc(2.0 * normalized_cutoff * offset)
}

fn design_windowed<F>(
    num_taps: usize,
    window: FirWindow,
    ideal_response: F,
) -> Vec<f64>
where
    F: Fn(usize, f64) -> f64,
{
    let centre = usize_to_f64(num_taps - 1) / 2.0;
    let mut taps = Vec::with_capacity(num_taps);

    for n in 0..num_taps {
        let offset = usize_to_f64(n) - centre;
        let coefficient =
            ideal_response(n, offset) * window.value(n, num_taps);

        taps.push(coefficient);
    }

    taps
}

fn response_magnitude(
    taps: &[f64],
    frequency_hz: f64,
    sample_rate: f64,
) -> f64 {
    let omega =
        std::f64::consts::TAU * frequency_hz / sample_rate;

    let mut real = 0.0;
    let mut imaginary = 0.0;

    for (n, &tap) in taps.iter().enumerate() {
        let phase = -omega * usize_to_f64(n);

        real += tap * phase.cos();
        imaginary += tap * phase.sin();
    }

    (real * real + imaginary * imaginary).sqrt()
}

fn normalize_at_frequency(
    taps: &mut [f64],
    frequency_hz: f64,
    sample_rate: f64,
) {
    let gain = response_magnitude(taps, frequency_hz, sample_rate);

    assert!(
        gain.is_finite() && gain > f64::EPSILON,
        "FIR design produced zero or non-finite normalization gain"
    );

    for tap in taps.iter_mut() {
        *tap /= gain;
    }
}

/// Designs a linear-phase low-pass FIR filter using the window method.
///
/// `cutoff_hz` is the half-amplitude cutoff frequency. The returned
/// coefficients are normalised for unity gain at DC.
///
/// Both odd and even tap counts are supported.
///
/// # Panics
///
/// Panics if:
///
/// - `num_taps == 0`;
/// - `sample_rate` is not finite and positive;
/// - `cutoff_hz` is not strictly between zero and Nyquist;
/// - the selected window parameters are invalid.
#[must_use]
pub fn firwin_lowpass(
    num_taps: usize,
    cutoff_hz: f64,
    sample_rate: f64,
    window: FirWindow,
) -> Vec<f64> {
    validate_common(num_taps, sample_rate);
    validate_cutoff(cutoff_hz, sample_rate);

    let mut taps = design_windowed(
        num_taps,
        window,
        |_, offset| ideal_lowpass(offset, cutoff_hz, sample_rate),
    );

    normalize_at_frequency(&mut taps, 0.0, sample_rate);

    taps
}

/// Designs a linear-phase high-pass FIR filter using the window method.
///
/// The returned coefficients are normalised for unity gain at Nyquist.
///
/// A high-pass filter whose passband reaches Nyquist requires an odd number
/// of taps. An even-length symmetric FIR is a Type-II filter and necessarily
/// has zero response at Nyquist.
///
/// # Panics
///
/// Panics if:
///
/// - `num_taps == 0`;
/// - `num_taps` is even;
/// - `sample_rate` is not finite and positive;
/// - `cutoff_hz` is not strictly between zero and Nyquist;
/// - the selected window parameters are invalid.
#[must_use]
pub fn firwin_highpass(
    num_taps: usize,
    cutoff_hz: f64,
    sample_rate: f64,
    window: FirWindow,
) -> Vec<f64> {
    validate_common(num_taps, sample_rate);
    validate_cutoff(cutoff_hz, sample_rate);

    assert!(
        num_taps % 2 == 1,
        "window-method high-pass FIR requires an odd number of taps"
    );

    let centre_index = num_taps / 2;

    let mut taps = design_windowed(
        num_taps,
        window,
        |n, offset| {
            let impulse = if n == centre_index { 1.0 } else { 0.0 };

            impulse - ideal_lowpass(offset, cutoff_hz, sample_rate)
        },
    );

    normalize_at_frequency(
        &mut taps,
        sample_rate / 2.0,
        sample_rate,
    );

    taps
}

/// Designs a linear-phase band-pass FIR filter using the window method.
///
/// The returned coefficients are normalised for unity gain at the centre of
/// the requested passband.
///
/// Both odd and even tap counts are supported.
///
/// # Panics
///
/// Panics if:
///
/// - `num_taps == 0`;
/// - `sample_rate` is not finite and positive;
/// - either cutoff is outside `(0, Nyquist)`;
/// - `lower_cutoff_hz >= upper_cutoff_hz`;
/// - the selected window parameters are invalid.
#[must_use]
pub fn firwin_bandpass(
    num_taps: usize,
    lower_cutoff_hz: f64,
    upper_cutoff_hz: f64,
    sample_rate: f64,
    window: FirWindow,
) -> Vec<f64> {
    validate_common(num_taps, sample_rate);

    validate_band(
        lower_cutoff_hz,
        upper_cutoff_hz,
        sample_rate,
    );

    let mut taps = design_windowed(
        num_taps,
        window,
        |_, offset| {
            ideal_lowpass(
                offset,
                upper_cutoff_hz,
                sample_rate,
            ) - ideal_lowpass(
                offset,
                lower_cutoff_hz,
                sample_rate,
            )
        },
    );

    let centre_frequency =
        (lower_cutoff_hz + upper_cutoff_hz) / 2.0;

    normalize_at_frequency(
        &mut taps,
        centre_frequency,
        sample_rate,
    );

    taps
}

/// Designs a linear-phase band-stop FIR filter using the window method.
///
/// The returned coefficients are normalised for unity gain at DC.
///
/// Because the upper passband extends to Nyquist, the filter requires an odd
/// number of taps.
///
/// # Panics
///
/// Panics if:
///
/// - `num_taps == 0`;
/// - `num_taps` is even;
/// - `sample_rate` is not finite and positive;
/// - either cutoff is outside `(0, Nyquist)`;
/// - `lower_cutoff_hz >= upper_cutoff_hz`;
/// - the selected window parameters are invalid.
#[must_use]
pub fn firwin_bandstop(
    num_taps: usize,
    lower_cutoff_hz: f64,
    upper_cutoff_hz: f64,
    sample_rate: f64,
    window: FirWindow,
) -> Vec<f64> {
    validate_common(num_taps, sample_rate);

    validate_band(
        lower_cutoff_hz,
        upper_cutoff_hz,
        sample_rate,
    );

    assert!(
        num_taps % 2 == 1,
        "window-method band-stop FIR requires an odd number of taps"
    );

    let centre_index = num_taps / 2;

    let mut taps = design_windowed(
        num_taps,
        window,
        |n, offset| {
            let impulse = if n == centre_index { 1.0 } else { 0.0 };

            impulse
                + ideal_lowpass(
                offset,
                lower_cutoff_hz,
                sample_rate,
            )
                - ideal_lowpass(
                offset,
                upper_cutoff_hz,
                sample_rate,
            )
        },
    );

    normalize_at_frequency(&mut taps, 0.0, sample_rate);

    taps
}

/// Designs an even-symmetry linear-phase FIR filter using the
/// Parks-McClellan Remez exchange algorithm.
///
/// `bands_hz` contains pairs of band edges:
///
/// `[band_0_start, band_0_end, band_1_start, band_1_end, ...]`.
///
/// `desired` contains one constant desired amplitude for each band.
///
/// `weights` contains one positive error weight for each band. Higher weight
/// places greater emphasis on reducing the approximation error in that band.
///
/// For example, a low-pass design may use:
///
/// ```text
/// bands_hz = [0, passband_end, stopband_start, Nyquist]
/// desired  = [1, 0]
/// weights  = [1, 10]
/// ```
///
/// Frequencies supplied to this public interface are in Hz. They are converted
/// internally into the `[0, 0.5]` cycles-per-sample frequency convention used
/// by the Parks-McClellan implementation.
///
/// # Panics
///
/// Panics if:
///
/// - `num_taps == 0`;
/// - `sample_rate` is not finite and positive;
/// - `bands_hz` is empty or does not contain complete start/end pairs;
/// - `desired` or `weights` does not contain exactly one value per band;
/// - a band is outside `[0, Nyquist]`;
/// - a band has zero or negative width;
/// - bands overlap or are not monotonically ordered;
/// - a desired amplitude is non-finite;
/// - a weight is non-finite or not positive;
/// - an even-length design requests non-zero response at Nyquist;
/// - the Parks-McClellan solver cannot produce a design.
#[must_use]
pub fn remez(
    num_taps: usize,
    bands_hz: &[f64],
    desired: &[f64],
    weights: &[f64],
    sample_rate: f64,
) -> Vec<f64> {
    validate_common(num_taps, sample_rate);

    assert!(
        !bands_hz.is_empty(),
        "Parks-McClellan design requires at least one band"
    );

    assert!(
        bands_hz.len() % 2 == 0,
        "Parks-McClellan bands must contain start/end pairs"
    );

    let band_count = bands_hz.len() / 2;

    assert_eq!(
        desired.len(),
        band_count,
        "Parks-McClellan desired response requires one value per band"
    );

    assert_eq!(
        weights.len(),
        band_count,
        "Parks-McClellan weights require one value per band"
    );

    let nyquist = sample_rate / 2.0;

    let mut band_settings = Vec::with_capacity(band_count);
    let mut previous_end = None;

    for band_index in 0..band_count {
        let start_hz = bands_hz[2 * band_index];
        let end_hz = bands_hz[2 * band_index + 1];
        let desired_amplitude = desired[band_index];
        let weight = weights[band_index];

        assert!(
            start_hz.is_finite()
                && end_hz.is_finite()
                && start_hz >= 0.0
                && end_hz <= nyquist,
            "Parks-McClellan band edges must be finite and within [0, Nyquist]"
        );

        assert!(
            start_hz < end_hz,
            "Parks-McClellan bands must have positive width"
        );

        if let Some(previous_end_hz) = previous_end {
            assert!(
                start_hz >= previous_end_hz,
                "Parks-McClellan bands must be monotonically ordered and non-overlapping"
            );
        }

        assert!(
            desired_amplitude.is_finite(),
            "Parks-McClellan desired amplitudes must be finite"
        );

        assert!(
            weight.is_finite() && weight > 0.0,
            "Parks-McClellan weights must be finite and greater than zero"
        );

        let normalized_start = start_hz / sample_rate;
        let normalized_end = end_hz / sample_rate;

        let setting: BandSetting<f64> = BandSetting::with_weight(
            normalized_start,
            normalized_end,
            constant(desired_amplitude),
            constant(weight),
        )
            .expect("validated Parks-McClellan band must be accepted");

        band_settings.push(setting);
        previous_end = Some(end_hz);
    }

    if num_taps % 2 == 0 {
        let final_band_end = bands_hz[bands_hz.len() - 1];
        let final_desired = desired[desired.len() - 1];

        assert!(
            final_band_end < nyquist
                || final_desired.abs() <= f64::EPSILON,
            "even-length symmetric FIR designs must have zero response at Nyquist"
        );
    }

    let parameters = pm_parameters(
        num_taps,
        &band_settings,
    )
        .expect("validated Parks-McClellan parameters must be accepted");

    let design = parks_mcclellan(&parameters)
        .expect("Parks-McClellan FIR design failed to converge");

    assert_eq!(
        design.impulse_response.len(),
        num_taps,
        "Parks-McClellan returned an unexpected number of taps"
    );

    design.impulse_response
}

#[cfg(test)]
mod tests {
    use super::*;
    use honeyeater_test::{assert_close, assert_spectral_mask, npy};
    use std::path::PathBuf;

    fn vector_path(filename: &str) -> PathBuf {
        let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        path.push("tests");
        path.push("vectors");
        path.push("fir_design");
        path.push(filename);

        path
    }

    fn frequency_response_db(
        taps: &[f64],
        bins: usize,
        sample_rate: f64,
    ) -> Vec<f64> {
        let nyquist = sample_rate / 2.0;
        let denominator =
            usize_to_f64(bins - 1);

        let mut response = Vec::with_capacity(bins);

        for bin in 0..bins {
            let frequency =
                nyquist * usize_to_f64(bin) / denominator;

            let magnitude =
                response_magnitude(taps, frequency, sample_rate);

            response.push(
                20.0 * magnitude.max(f64::MIN_POSITIVE).log10(),
            );
        }

        response
    }

    #[test]
    fn test_firwin_lowpass_matches_scipy_oracle() {
        let expected = npy::load_f64(
            &vector_path("lowpass_hamming_63.npy"),
        )
            .expect("failed to load low-pass FIR design reference vector");

        let actual = firwin_lowpass(
            63,
            100_000.0,
            1_000_000.0,
            FirWindow::Hamming,
        );

        assert_close!(
            actual,
            expected,
            rtol = 1e-11,
            atol = 1e-13,
        );
    }

    #[test]
    fn test_firwin_highpass_matches_scipy_oracle() {
        let expected = npy::load_f64(
            &vector_path("highpass_kaiser_63.npy"),
        )
            .expect("failed to load high-pass FIR design reference vector");

        let actual = firwin_highpass(
            63,
            150_000.0,
            1_000_000.0,
            FirWindow::Kaiser(7.5),
        );

        assert_close!(
            actual,
            expected,
            rtol = 1e-11,
            atol = 1e-13,
        );
    }

    #[test]
    fn test_firwin_bandpass_matches_scipy_oracle() {
        let expected = npy::load_f64(
            &vector_path("bandpass_hann_64.npy"),
        )
            .expect("failed to load band-pass FIR design reference vector");

        let actual = firwin_bandpass(
            64,
            100_000.0,
            200_000.0,
            1_000_000.0,
            FirWindow::Hann,
        );

        assert_close!(
            actual,
            expected,
            rtol = 1e-11,
            atol = 1e-13,
        );
    }

    #[test]
    fn test_firwin_bandstop_matches_scipy_oracle() {
        let expected = npy::load_f64(
            &vector_path("bandstop_blackmanharris_63.npy"),
        )
            .expect("failed to load band-stop FIR design reference vector");

        let actual = firwin_bandstop(
            63,
            100_000.0,
            200_000.0,
            1_000_000.0,
            FirWindow::BlackmanHarris,
        );

        assert_close!(
            actual,
            expected,
            rtol = 1e-11,
            atol = 1e-13,
        );
    }

    #[test]
    fn test_remez_lowpass_meets_spectral_mask() {
        const SAMPLE_RATE: f64 = 1_000_000.0;
        const PASSBAND_END: f64 = 100_000.0;
        const STOPBAND_START: f64 = 150_000.0;
        const BINS: usize = 8193;

        let taps = remez(
            81,
            &[
                0.0,
                PASSBAND_END,
                STOPBAND_START,
                SAMPLE_RATE / 2.0,
            ],
            &[1.0, 0.0],
            &[1.0, 10.0],
            SAMPLE_RATE,
        );

        let response =
            frequency_response_db(&taps, BINS, SAMPLE_RATE);

        let mut lower =
            vec![f64::NEG_INFINITY; BINS];

        let mut upper =
            vec![f64::INFINITY; BINS];

        let nyquist = SAMPLE_RATE / 2.0;
        let denominator = usize_to_f64(BINS - 1);

        for bin in 0..BINS {
            let frequency =
                nyquist * usize_to_f64(bin) / denominator;

            if frequency <= PASSBAND_END {
                lower[bin] = -0.1;
                upper[bin] = 0.1;
            } else if frequency >= STOPBAND_START {
                upper[bin] = -50.0;
            }
        }

        assert_spectral_mask!(
            response,
            lower = lower,
            upper = upper,
        );
    }

    #[test]
    #[should_panic(
        expected = "window-method high-pass FIR requires an odd number of taps"
    )]
    fn test_firwin_highpass_rejects_even_tap_count() {
        let _ = firwin_highpass(
            64,
            100_000.0,
            1_000_000.0,
            FirWindow::Hamming,
        );
    }

    #[test]
    #[should_panic(
        expected = "window-method band-stop FIR requires an odd number of taps"
    )]
    fn test_firwin_bandstop_rejects_even_tap_count() {
        let _ = firwin_bandstop(
            64,
            100_000.0,
            200_000.0,
            1_000_000.0,
            FirWindow::Hamming,
        );
    }

    #[test]
    #[should_panic(
        expected = "even-length symmetric FIR designs must have zero response at Nyquist"
    )]
    fn test_remez_rejects_even_highpass() {
        let _ = remez(
            64,
            &[
                0.0,
                100_000.0,
                150_000.0,
                500_000.0,
            ],
            &[0.0, 1.0],
            &[10.0, 1.0],
            1_000_000.0,
        );
    }
}