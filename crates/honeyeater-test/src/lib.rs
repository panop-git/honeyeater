//! Cross-validation helpers and tolerance assertion macros for honeyeater's
//! own test suite. Not published; not intended as a general testing library.
//!
//! ## The seven assertion macros
//!
//! The roadmap (`docs/roadmap.md`, "Tolerance vocabulary") commits honeyeater
//! to a small fixed set of tolerance measures, intended to be used
//! consistently across the codebase rather than letting ad-hoc thresholds
//! accumulate per test file. The seven macros below are the implementation
//! of that vocabulary:
//!
//! | Macro | Predicate | Intended for |
//! |---|---|---|
//! | [`assert_close!`] | `\|a − b\| ≤ atol + rtol·\|b\|`, elementwise | pointwise array comparison: FIR/IIR output, FFT bins, resampler output, window taps |
//! | [`assert_snr_db!`] | `10·log₁₀(Σ\|ref\|² / Σ\|ref − actual\|²) ≥ min_db` | structured signal-vs-reference: filters, FFT round-trips, resamplers, modulators, AGC |
//! | [`assert_bit_exact!`] | exact equality at the byte or bit level | FEC encoders, fixed-point kernels, CRCs, scramblers |
//! | [`assert_spectral_mask!`] | each bin within `[lower(f), upper(f)]` dB | filter design verification, transmit-spectrum compliance |
//! | [`assert_ber_at_ebn0!`] | BER ≤ target at stated Eb/N0 over Monte-Carlo trials | FEC decoders, demodulator slicers |
//! | [`assert_parseval!`] | one-sided PSD integral ≈ time-domain energy | spectral estimators |
//! | [`assert_distribution_ks!`] | Kolmogorov-Smirnov test against target CDF | PRNGs, AWGN generators, noise sources |
//!
//! See `docs/roadmap.md` for the default thresholds per module class and the
//! rationale behind the chosen predicates.
//!
//! ## Reference vectors and cross-validation
//!
//! Two helpers complete the harness:
//!
//! - [`npy::load_f32`] / [`npy::load_f64`] / [`npy::load_complex_f32`] —
//!   load `.npy` reference vectors committed to `tests/vectors/`.
//! - [`scipy::run`] — call out to a Python interpreter with scipy installed
//!   for live cross-validation when committing a vector is overkill.
//!
//! Both are stubs at Phase 0 (signatures defined, bodies deferred to first
//! use). They are documented here so the test-writing recipe is in one
//! place when Phase 1 starts.

#![forbid(unsafe_code)]

mod macros;

pub mod npy;
pub mod scipy;
pub mod stats;
