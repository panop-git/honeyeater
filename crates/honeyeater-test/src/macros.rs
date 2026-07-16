//! The seven tolerance assertion macros.
//!
//! Each macro takes the form `assert_<measure>!(actual, expected, params...)`
//! and panics with a diagnostic message if the predicate fails. Diagnostic
//! messages include the failing index (where applicable), the measured
//! value, the threshold, and the slack — so a CI failure is actionable
//! without re-running with a debugger attached.

/// Elementwise close-comparison: `|a − b| ≤ atol + rtol·|b|`.
///
/// This is the numpy / MATLAB convention for "are these arrays equal up to
/// floating-point slop." It is the primary measure for pointwise array
/// comparison: FIR/IIR output, FFT bins, resampler output, window taps.
///
/// # Parameters
///
/// - `$actual`: the array under test, anything dereferencing to `&[f64]` or
///   `&[f32]`.
/// - `$expected`: the reference array, same element type and length.
/// - `rtol = <expr>`: relative tolerance (multiplier on `|expected|`).
/// - `atol = <expr>`: absolute tolerance (floor for values near zero).
///
/// Both `rtol` and `atol` are required. Picking them is a design decision per
/// kernel; defaults per module class are documented in `docs/roadmap.md`,
/// "Default thresholds per module class". For window-function tests scipy
/// uses `rtol = 1e-12, atol = 1e-15`.
///
/// # Panics
///
/// Panics with the first failing index, the two values, the measured
/// `|a − b|`, and the threshold `atol + rtol·|b|`. The arrays must have
/// equal length; a length mismatch is itself a panic.
#[macro_export]
macro_rules! assert_close {
    ($actual:expr, $expected:expr, rtol = $rtol:expr, atol = $atol:expr $(,)?) => {{
        let actual = &$actual;
        let expected = &$expected;
        let rtol: f64 = $rtol;
        let atol: f64 = $atol;
        assert_eq!(
            actual.len(),
            expected.len(),
            "assert_close!: length mismatch: actual.len() = {}, expected.len() = {}",
            actual.len(),
            expected.len(),
        );
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            let a_f = f64::from(*a);
            let e_f = f64::from(*e);
            let diff = (a_f - e_f).abs();
            let threshold = atol + rtol * e_f.abs();
            assert!(
                diff <= threshold,
                "assert_close! failed at index {i}: \
                 actual = {a_f}, expected = {e_f}, |a-b| = {diff:e}, \
                 threshold = atol + rtol*|b| = {atol:e} + {rtol:e}*{:e} = {threshold:e}",
                e_f.abs(),
            );
        }
    }};
}

/// Signal-to-noise ratio in dB against a reference array, with a minimum
/// threshold: `10·log₁₀(Σ|ref|² / Σ|ref − actual|²) ≥ min_db`.
///
/// This is the structured-signal measure: it asks "how much of the reference
/// signal's energy survives the perturbation introduced by the kernel under
/// test." Used for filters, FFT round-trips, resamplers, modulators, AGC,
/// and other kernels where the output is a transformed but still-structured
/// version of a known input.
///
/// # Parameters
///
/// - `$actual`: array under test.
/// - `$reference`: the reference signal (typically the input, or an analytic
///   ground truth).
/// - `min_db = <expr>`: minimum acceptable SNR in dB.
///
/// Defaults from `docs/roadmap.md`: 120 dB for f64 FFT, 60 dB for f32 FFT,
/// 100 dB for f64 FIR output, 80 dB for f64 IIR/polyphase resampler, 100 dB
/// for f64 linear modulators.
///
/// # Panics
///
/// Panics with the measured SNR in dB and the threshold. Panics if the
/// arrays differ in length, if the reference has zero energy (SNR is
/// undefined), or if the error is identically zero (SNR is `+∞`, which
/// always satisfies any finite threshold but is reported anyway).
#[macro_export]
macro_rules! assert_snr_db {
    ($actual:expr, $reference:expr, min_db = $min_db:expr $(,)?) => {{
        let actual = &$actual;
        let reference = &$reference;
        let min_db: f64 = $min_db;
        assert_eq!(
            actual.len(),
            reference.len(),
            "assert_snr_db!: length mismatch: actual.len() = {}, reference.len() = {}",
            actual.len(),
            reference.len(),
        );
        let (ref_energy, err_energy) =
            actual
                .iter()
                .zip(reference.iter())
                .fold((0.0_f64, 0.0_f64), |(re, ee), (a, r)| {
                    let a_f = f64::from(*a);
                    let r_f = f64::from(*r);
                    (re + r_f * r_f, ee + (a_f - r_f).powi(2))
                });
        assert!(
            ref_energy > 0.0,
            "assert_snr_db!: reference has zero energy; SNR is undefined",
        );
        let snr_db = if err_energy == 0.0 {
            f64::INFINITY
        } else {
            10.0 * (ref_energy / err_energy).log10()
        };
        assert!(
            snr_db >= min_db,
            "assert_snr_db! failed: SNR = {snr_db:.3} dB, threshold = {min_db:.3} dB \
             (ref_energy = {ref_energy:e}, err_energy = {err_energy:e})",
        );
    }};
}

/// Exact equality at the byte or bit level.
///
/// Used for kernels with a deterministic output representation: FEC
/// encoders, fixed-point kernels (where the integer arithmetic is exact),
/// CRCs, scramblers. For these, "close enough" is the wrong measure — the
/// output must match the spec exactly.
///
/// # Parameters
///
/// - `$actual`: array under test.
/// - `$expected`: the reference array. Both must have the same element type
///   implementing [`PartialEq`] and [`core::fmt::Debug`].
///
/// # Panics
///
/// Panics on length mismatch, or on the first differing index with both
/// values printed via `Debug`. For arrays of bytes the message also prints
/// a compact hex dump of the surrounding window for context.
#[macro_export]
macro_rules! assert_bit_exact {
    ($actual:expr, $expected:expr $(,)?) => {{
        let actual = &$actual;
        let expected = &$expected;
        assert_eq!(
            actual.len(),
            expected.len(),
            "assert_bit_exact!: length mismatch: actual.len() = {}, expected.len() = {}",
            actual.len(),
            expected.len(),
        );
        for (i, (a, e)) in actual.iter().zip(expected.iter()).enumerate() {
            assert!(
                a == e,
                "assert_bit_exact! failed at index {i}: actual = {a:?}, expected = {e:?}",
            );
        }
    }};
}

/// Spectral-mask compliance: each bin's magnitude in dB must lie within
/// `[lower(f), upper(f)]`.
///
/// Used to validate filter designs (passband ripple, stopband attenuation)
/// and transmit-spectrum compliance (e.g. ETSI / FCC out-of-band emission
/// limits).
///
/// # Parameters
///
/// - `$bins_db`: slice of bin magnitudes in dB (one per frequency bin).
/// - `$lower_db`: slice of lower bounds in dB, same length. Use
///   `f64::NEG_INFINITY` for "no lower bound at this frequency."
/// - `$upper_db`: slice of upper bounds in dB, same length. Use
///   `f64::INFINITY` for "no upper bound at this frequency."
///
/// All three slices must have the same length. The caller is responsible
/// for converting from linear magnitude to dB before passing in.
///
/// # Panics
///
/// Panics with the first failing bin index, the bin's measured dB level,
/// and the violated bound.
#[macro_export]
macro_rules! assert_spectral_mask {
    ($bins_db:expr, lower = $lower_db:expr, upper = $upper_db:expr $(,)?) => {{
        let bins = &$bins_db;
        let lower = &$lower_db;
        let upper = &$upper_db;
        assert_eq!(
            bins.len(),
            lower.len(),
            "assert_spectral_mask!: bins/lower length mismatch ({} vs {})",
            bins.len(),
            lower.len(),
        );
        assert_eq!(
            bins.len(),
            upper.len(),
            "assert_spectral_mask!: bins/upper length mismatch ({} vs {})",
            bins.len(),
            upper.len(),
        );
        for (i, ((b, lo), hi)) in bins.iter().zip(lower.iter()).zip(upper.iter()).enumerate() {
            let b_f: f64 = f64::from(*b);
            let lo_f: f64 = f64::from(*lo);
            let hi_f: f64 = f64::from(*hi);
            assert!(
                b_f >= lo_f,
                "assert_spectral_mask! failed at bin {i}: \
                 measured = {b_f:.3} dB, lower bound = {lo_f:.3} dB",
            );
            assert!(
                b_f <= hi_f,
                "assert_spectral_mask! failed at bin {i}: \
                 measured = {b_f:.3} dB, upper bound = {hi_f:.3} dB",
            );
        }
    }};
}

/// Bit-error-rate at a stated Eb/N0, over a Monte-Carlo trial: `BER ≤ target`.
///
/// This is the only sensible measure for iterative soft-decision FEC
/// decoders (turbo, LDPC, polar SCL), where bit-exact comparison against a
/// reference implementation is impossible because implementations diverge
/// on quantisation and scheduling. Also used for demodulator-slicer
/// validation against closed-form AWGN BER formulas (the Q-function family).
///
/// # Parameters
///
/// - `$errors`: number of bit errors observed.
/// - `$total_bits`: total bits transmitted in the trial.
/// - `target_ber = <expr>`: maximum acceptable BER.
/// - `ebn0_db = <expr>`: Eb/N0 in dB at which the trial was run (printed in
///   the panic message for context; does not affect the predicate).
///
/// Defaults from `docs/roadmap.md`: 0.2 dB tolerance at the waterfall and
/// 0.5 dB in the error floor, against AFF3CT reference curves for LDPC and
/// CCSDS published plots for AR4JA.
///
/// # Panics
///
/// Panics with the measured BER, the target, and the Eb/N0 at which the
/// trial was run.
#[macro_export]
macro_rules! assert_ber_at_ebn0 {
    (
        $errors:expr,
        $total_bits:expr,
        target_ber = $target:expr,
        ebn0_db = $ebn0:expr $(,)?
    ) => {{
        let errors: u64 = $errors;
        let total_bits: u64 = $total_bits;
        let target: f64 = $target;
        let ebn0_db: f64 = $ebn0;
        assert!(
            total_bits > 0,
            "assert_ber_at_ebn0!: total_bits must be > 0",
        );
        // The u64→f64 cast loses precision above 2^52 bits, which is
        // ~4.5 PB of trial data — outside any realistic Monte-Carlo trial
        // we'd run in a test. For practical BER ranges (10⁻¹ down to 10⁻¹⁵)
        // the f64 division gives the correct result to far better than the
        // 0.2 dB / 0.5 dB tolerances the roadmap specifies.
        #[allow(clippy::cast_precision_loss)]
        let ber = errors as f64 / total_bits as f64;
        assert!(
            ber <= target,
            "assert_ber_at_ebn0! failed: BER = {ber:e} ({errors}/{total_bits}), \
             target = {target:e}, Eb/N0 = {ebn0_db:.2} dB",
        );
    }};
}

/// Parseval-energy equivalence: the one-sided PSD integral equals the
/// time-domain signal energy, within a relative tolerance.
///
/// This is the measure that resolves the scipy / MATLAB / Octave Welch
/// PSD-scaling trap: different libraries scale their PSD differently, and a
/// kernel that gets the bin-width or window-correction wrong produces a
/// PSD that looks right at first glance but fails to integrate to the true
/// signal energy.
///
/// # Parameters
///
/// - `$psd`: slice of PSD values (one-sided, units of power per Hz).
/// - `$bin_width_hz`: the bin width in Hz (`fs / nfft` for unwindowed PSD;
///   the caller is responsible for any window-correction factor).
/// - `$signal_energy`: the reference signal's time-domain energy
///   (`(1/fs)·Σ|x|²`).
/// - `rtol = <expr>`: relative tolerance on the integrated/reference ratio.
///
/// # Panics
///
/// Panics with the integrated energy, the reference energy, the ratio, and
/// the tolerance.
#[macro_export]
macro_rules! assert_parseval {
    (
        $psd:expr,
        bin_width_hz = $bin_width:expr,
        signal_energy = $energy:expr,
        rtol = $rtol:expr $(,)?
    ) => {{
        let psd = &$psd;
        let bin_width: f64 = $bin_width;
        let signal_energy: f64 = $energy;
        let rtol: f64 = $rtol;
        assert!(
            bin_width > 0.0,
            "assert_parseval!: bin_width_hz must be > 0",
        );
        assert!(
            signal_energy > 0.0,
            "assert_parseval!: signal_energy must be > 0",
        );
        let integrated: f64 = psd.iter().map(|v| f64::from(*v)).sum::<f64>() * bin_width;
        let ratio = integrated / signal_energy;
        let diff = (ratio - 1.0).abs();
        assert!(
            diff <= rtol,
            "assert_parseval! failed: integrated PSD = {integrated:e}, \
             signal energy = {signal_energy:e}, ratio = {ratio:.6}, \
             |ratio - 1| = {diff:e}, rtol = {rtol:e}",
        );
    }};
}

/// Kolmogorov-Smirnov goodness-of-fit test against a target CDF.
///
/// Used for PRNGs, AWGN generators, and other noise sources where the
/// implementation's output should match a target distribution.
///
/// Per `docs/roadmap.md`, "Statistical tests in CI": this macro is intended
/// for CI use with a **fixed seed**, where it acts as a deterministic vector
/// regression — flake-free, no real distributional claim. The true
/// distributional question belongs in a separate nightly job that runs many
/// independent seeds and checks uniformity of the resulting p-values.
///
/// # Parameters
///
/// - `$samples`: slice of samples drawn from the implementation under test.
/// - `cdf = <closure>`: a closure `Fn(f64) -> f64` returning the target
///   CDF's value at a point.
/// - `alpha = <expr>`: significance level. The macro computes the critical
///   D-statistic from `alpha` and the sample size.
///
/// # Panics
///
/// Panics with the measured D-statistic and the critical value. Panics if
/// the sample slice is empty.
#[macro_export]
macro_rules! assert_distribution_ks {
    (
        $samples:expr,
        cdf = $cdf:expr,
        alpha = $alpha:expr $(,)?
    ) => {{
        let samples: &[f64] = &$samples;
        let cdf: fn(f64) -> f64 = $cdf;
        let alpha: f64 = $alpha;
        let (d_stat, d_critical) = $crate::stats::ks_one_sample(samples, cdf, alpha);
        assert!(
            d_stat <= d_critical,
            "assert_distribution_ks! failed: D = {d_stat:.6}, \
             critical = {d_critical:.6} (alpha = {alpha}, n = {})",
            samples.len(),
        );
    }};
}

#[cfg(test)]
mod tests {
    /// `assert_close!` accepts arrays that match exactly.
    #[test]
    fn assert_close_exact_match() {
        let a: [f64; 3] = [1.0, 2.0, 3.0];
        let b: [f64; 3] = [1.0, 2.0, 3.0];
        crate::assert_close!(a, b, rtol = 1e-12, atol = 1e-15);
    }

    /// `assert_close!` accepts arrays within tolerance.
    #[test]
    fn assert_close_within_tol() {
        let a: [f64; 3] = [1.0, 2.0, 3.0];
        let b: [f64; 3] = [1.0 + 1e-13, 2.0 - 1e-13, 3.0 + 1e-13];
        crate::assert_close!(a, b, rtol = 1e-12, atol = 1e-15);
    }

    /// `assert_close!` rejects arrays outside tolerance.
    #[test]
    #[should_panic(expected = "assert_close! failed")]
    fn assert_close_rejects_outside_tol() {
        let a: [f64; 3] = [1.0, 2.0, 3.0];
        let b: [f64; 3] = [1.0, 2.5, 3.0];
        crate::assert_close!(a, b, rtol = 1e-12, atol = 1e-15);
    }

    /// `assert_snr_db!` accepts identical signals (SNR = +inf).
    #[test]
    fn assert_snr_identical() {
        let x: [f64; 4] = [1.0, -1.0, 1.0, -1.0];
        crate::assert_snr_db!(x, x, min_db = 200.0);
    }

    /// `assert_snr_db!` rejects when the SNR is below the threshold.
    #[test]
    #[should_panic(expected = "assert_snr_db! failed")]
    fn assert_snr_rejects_low() {
        let r: [f64; 2] = [1.0, 1.0];
        let a: [f64; 2] = [0.5, 0.5];
        crate::assert_snr_db!(a, r, min_db = 60.0);
    }

    /// `assert_bit_exact!` accepts identical byte arrays.
    #[test]
    fn assert_bit_exact_match() {
        let a: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
        let b: [u8; 4] = [0xDE, 0xAD, 0xBE, 0xEF];
        crate::assert_bit_exact!(a, b);
    }

    /// `assert_bit_exact!` rejects differing byte arrays.
    #[test]
    #[should_panic(expected = "assert_bit_exact! failed")]
    fn assert_bit_exact_rejects_diff() {
        let a: [u8; 2] = [0xDE, 0xAD];
        let b: [u8; 2] = [0xDE, 0xAE];
        crate::assert_bit_exact!(a, b);
    }

    /// `assert_spectral_mask!` accepts bins inside their bounds.
    #[test]
    fn assert_spectral_mask_inside() {
        let bins: [f64; 3] = [-3.0, -3.0, -50.0];
        let lower: [f64; 3] = [f64::NEG_INFINITY, f64::NEG_INFINITY, f64::NEG_INFINITY];
        let upper: [f64; 3] = [0.0, 0.0, -40.0];
        crate::assert_spectral_mask!(bins, lower = lower, upper = upper);
    }

    /// `assert_spectral_mask!` rejects when an upper bound is exceeded.
    #[test]
    #[should_panic(expected = "assert_spectral_mask! failed")]
    fn assert_spectral_mask_rejects_upper() {
        let bins: [f64; 1] = [3.0];
        let lower: [f64; 1] = [f64::NEG_INFINITY];
        let upper: [f64; 1] = [0.0];
        crate::assert_spectral_mask!(bins, lower = lower, upper = upper);
    }

    /// `assert_ber_at_ebn0!` accepts zero errors.
    #[test]
    fn assert_ber_zero_errors() {
        crate::assert_ber_at_ebn0!(0_u64, 1_000_000_u64, target_ber = 1e-6, ebn0_db = 5.0);
    }

    /// `assert_ber_at_ebn0!` rejects when above target.
    #[test]
    #[should_panic(expected = "assert_ber_at_ebn0! failed")]
    fn assert_ber_rejects_above_target() {
        crate::assert_ber_at_ebn0!(100_u64, 1_000_u64, target_ber = 1e-3, ebn0_db = 3.0);
    }

    /// `assert_parseval!` accepts an exact match.
    #[test]
    fn assert_parseval_exact() {
        // Two PSD bins each at value 1.0, bin width 0.5 Hz, total integrated energy 1.0.
        let psd: [f64; 2] = [1.0, 1.0];
        crate::assert_parseval!(psd, bin_width_hz = 0.5, signal_energy = 1.0, rtol = 1e-9);
    }

    /// `assert_parseval!` rejects when the ratio exceeds the tolerance.
    #[test]
    #[should_panic(expected = "assert_parseval! failed")]
    fn assert_parseval_rejects() {
        let psd: [f64; 2] = [1.0, 1.0];
        crate::assert_parseval!(psd, bin_width_hz = 0.5, signal_energy = 2.0, rtol = 1e-3);
    }
}
