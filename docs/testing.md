# Testing

honeyeater is test-driven: every [kernel](glossary.md#kernel) ships with a test against a named [oracle](glossary.md#oracle) — a trusted reference (an established library, or correct values from a standard) that says what the right answer is. This page documents the test harness for kernel authors. Unfamiliar term? The [Glossary](glossary.md) has a one-line definition for each.

The page has three sections:

- **[Overview](#overview)** — what the harness is. Read this if honeyeater is new to you.
- **[How-to guides](#how-to-guides)** — task-shaped recipes ("how do I test a filter?"). Read this when writing a specific test.
- **[Reference](#reference)** — exhaustive description of each macro and helper. Read this when looking up a parameter or threshold.

---

## Overview

honeyeater commits to a small, fixed vocabulary of [tolerance](glossary.md#tolerance-atol--rtol) measures — precise definitions of how close a result must be to count as correct. Every kernel is validated against a named oracle (an established reference implementation, or bit-exact vectors from a published standard) using one of seven assertion macros:

| Macro | Predicate |
|---|---|
| `assert_close!` | elementwise `\|a − b\| ≤ atol + rtol·\|b\|` |
| `assert_snr_db!` | `10·log₁₀(Σ\|ref\|² / Σ\|ref − actual\|²) ≥ min_db` |
| `assert_bit_exact!` | exact equality at the byte or bit level |
| `assert_spectral_mask!` | each bin within `[lower(f), upper(f)]` dB |
| `assert_ber_at_ebn0!` | `BER ≤ target` at stated Eb/N0 |
| `assert_parseval!` | one-sided PSD integral ≈ time-domain energy |
| `assert_distribution_ks!` | Kolmogorov-Smirnov against a target CDF |

These are the only tolerance predicates honeyeater uses across the codebase. Percentage tolerance is **not** in the set — it breaks on zero crossings and is insensitive to dynamic range. Ad-hoc thresholds invented per-test are discouraged.

The harness lives in the dev-only `honeyeater-test` crate. It is not published; you do not depend on it in production code. The published `honeyeater` crate has no test dependencies.

The seven macros are implemented and tested. The `.npy` reference-vector loader (`honeyeater_test::npy`) and the scipy-subprocess runner (`honeyeater_test::scipy`) are currently signature-only stubs.

### Importing the harness

```toml
[dev-dependencies]
honeyeater-test = { path = "path/to/honeyeater/crates/honeyeater-test" }
```

---

## How-to guides

Task-shaped recipes for common testing situations. Kernels named in the examples (`my_fir_filter`, `my_rs_encoder`, etc.) are placeholders — substitute your own.

### How to test a filter

Filters get tested two ways: against a precomputed reference vector for exactness, and against a known input signal for a property the filter should preserve.

```rust
use honeyeater_test::{assert_close, assert_snr_db, npy};
use std::path::Path;

#[test]
fn my_fir_filter_matches_scipy_reference() {
    let input = npy::load_f64(Path::new("tests/vectors/impulse_response_input.npy"));
    let expected = npy::load_f64(Path::new("tests/vectors/fir_lpf_64tap_output.npy"));

    let actual = my_fir_filter(&input, 64, 0.25);

    assert_close!(actual, expected, rtol = 1e-12, atol = 1e-15);
}

#[test]
fn my_fir_filter_preserves_in_band_signal_to_100_db() {
    let input = sinusoid(0.1, 1024);
    let output = my_fir_filter(&input, 64, 0.25);

    assert_snr_db!(output, input, min_db = 100.0);
}
```

Both tests belong in the suite. The reference vector catches deviations from scipy's `lfilter` output; the [SNR](glossary.md#snr-and-db) property (signal-to-noise ratio, in decibels) catches structural bugs (sign flip, off-by-one in tap indexing) that might still match the reference *if the reference is regenerated from the same bug*.

### How to test an FEC encoder

[FEC](glossary.md#fec-forward-error-correction) (forward error correction) encoders are deterministic and must match the spec byte-for-byte. Bit-exact comparison is the right predicate.

```rust
use honeyeater_test::assert_bit_exact;

#[test]
fn my_rs_encoder_matches_libfec_vector() {
    let message: Vec<u8> = std::fs::read("tests/vectors/rs_message.bin").unwrap();
    let expected: Vec<u8> = std::fs::read("tests/vectors/rs_codeword.bin").unwrap();

    let actual = my_rs_encoder(&message);

    assert_bit_exact!(actual, expected);
}
```

The reference vector here is a binary blob committed alongside the kernel. The generator that produced it lives in `tools/oracle-gen/`, not in the main crate's link graph, so the LGPL-licensed `libfec` never enters honeyeater's published dependencies.

### How to test an FEC decoder

Decoders — especially iterative soft-decision ones ([turbo, LDPC, polar-SCL](glossary.md#ldpc-turbo-polar-scl)) — cannot be bit-exact-tested because different implementations diverge on quantisation and scheduling. [BER](glossary.md#ber-bit-error-rate) (bit error rate — the fraction of bits the link gets wrong) at [Eb/N0](glossary.md#ebn0) (a normalised signal-to-noise measure) is the only meaningful measure.

```rust
use honeyeater_test::assert_ber_at_ebn0;

#[test]
fn my_ldpc_decoder_meets_waterfall_at_3_5_db() {
    let (errors, total_bits) = run_ldpc_trial(/* ebn0_db = */ 3.5, /* trials = */ 100_000);

    assert_ber_at_ebn0!(
        errors,
        total_bits,
        target_ber = 1e-5,
        ebn0_db = 3.5,
    );
}
```

The test name refers to the [waterfall](glossary.md#waterfall) — the steep part of the BER-versus-Eb/N0 curve, where a small gain in signal-to-noise sharply drops the error rate.

Trial size matters: targeting a BER of 10⁻⁵ requires order 10⁷ bits transmitted to see ~100 errors and have a stable estimate. Tests that demand much lower BERs at small trial sizes are not useful.

### How to test a PRNG or noise source

Use [Kolmogorov-Smirnov](glossary.md#kolmogorov-smirnov-ks-test) (a statistical test for "do these samples come from the distribution I claim?") against the target distribution — supplied as a [CDF](glossary.md#cdf-cumulative-distribution-function) (cumulative distribution function) — with a fixed seed. ([PRNG](glossary.md#prng): a pseudo-random number generator, which a fixed seed makes repeatable.)

```rust
use honeyeater_test::assert_distribution_ks;

#[test]
fn my_uniform_prng_passes_ks_at_alpha_001() {
    let samples = my_uniform_prng(/* seed = */ 0xC0FFEE, /* count = */ 10_000);

    let uniform_cdf = |x: f64| x.clamp(0.0, 1.0);
    assert_distribution_ks!(samples, cdf = uniform_cdf, alpha = 0.01);
}
```

The fixed seed makes this a **deterministic regression test**, not a statistical claim. See the [`assert_distribution_ks!` reference](#assert_distribution_ks) for the critical distinction between CI use and real statistical validation — getting this wrong leads to flaky tests.

### How to validate a filter design against a spectral mask

Filter designs are validated by *response shape*, not by individual sample equality. The mask sets per-bin upper and lower bounds in dB.

```rust
use honeyeater_test::assert_spectral_mask;

#[test]
fn my_fir_lpf_meets_design_spec() {
    let response_db = compute_frequency_response_db(&my_fir_lpf_64tap());

    let (lower, upper) = build_lpf_mask(
        /* passband_edge = */ 0.20,
        /* stopband_edge = */ 0.30,
        /* passband_ripple_db = */ 0.1,
        /* stopband_atten_db = */ 60.0,
    );

    assert_spectral_mask!(response_db, lower = lower, upper = upper);
}
```

Use `f64::NEG_INFINITY` and `f64::INFINITY` for "no bound here" in the appropriate ends of the spectrum. Most filter masks have constraints only on the upper bound in the stopband and only on the lower bound in the passband.

### How to validate a spectral estimator (Welch, periodogram, multitaper)

Spectral estimators have a notorious failure mode: getting the bin-width or window-correction factor wrong produces a [PSD](glossary.md#psd-power-spectral-density) (power spectral density — how the signal's power is spread across frequency) that looks plausible at first glance but doesn't integrate to the true signal energy. The [Parseval](glossary.md#parsevals-theorem) assertion — which checks that the energy adds up the same measured in time or in frequency — catches this exactly.

```rust
use honeyeater_test::assert_parseval;

#[test]
fn my_welch_psd_conserves_energy() {
    let signal = generate_test_signal();
    let psd = my_welch(&signal, /* nperseg = */ 256, /* overlap = */ 128);

    let fs = 1.0;
    let bin_width = fs / 256.0;
    let signal_energy: f64 = signal.iter().map(|x| x * x / fs).sum();

    assert_parseval!(
        psd,
        bin_width_hz = bin_width,
        signal_energy = signal_energy,
        rtol = 1e-6,
    );
}
```

### How to pick a threshold

Start from the per-class default table:

| Module class | Primary assertion | Threshold |
|---|---|---|
| FFT (f64) | `assert_snr_db!` | ≥ 120 dB |
| FFT (f32) | `assert_snr_db!` | ≥ 60 dB |
| FIR output (f64) | `assert_snr_db!` | ≥ 100 dB |
| FIR output (f32) | `assert_snr_db!` | ≥ 60 dB |
| FIR design | `assert_spectral_mask!` | passband ±0.1 dB; stopband per spec |
| IIR (f64) | `assert_snr_db!` vs scipy `lfilter` | ≥ 80 dB |
| Polyphase resampler | `assert_snr_db!` | ≥ 80 dB |
| Window functions | `assert_close!` | `rtol = 1e-12, atol = 1e-15` |
| Linear modulator (f64) | `assert_snr_db!` vs analytic reference | ≥ 100 dB |
| FEC encoder | `assert_bit_exact!` | byte equality |
| FEC decoder (iterative) | `assert_ber_at_ebn0!` | spec-dependent — 0.2 dB at the waterfall, 0.5 dB in the error floor |
| [EVM](glossary.md#evm-error-vector-magnitude) aggregate | percent | per the relevant 3GPP TS |
| AWGN / PRNG | `assert_distribution_ks!` + moment match | KS α = 0.01 with fixed seed; mean / variance within 3σ |
| AGC | `assert_snr_db!` + settling | within ±0.5 dB steady-state |

Loosen only when you can articulate why the kernel cannot meet the listed threshold. Legitimate reasons exist (narrowband filters with high Q, low-bit-width fixed-point kernels) but the burden of explanation is on the test author.

### How to write a cross-platform test without flakes

A test that passes on x86 may not pass on aarch64 without help. Three categories of variation cause failures, in order of how often they bite:

1. **[FMA](glossary.md#fma-fused-multiply-add) contraction.** The compiler may fuse `a * b + c` into a single fused multiply-add instruction, which is a different (more accurate) operation than the separate multiply and add. Test reference paths should set `-ffp-contract=off` or pin a no-FMA reference computation.
2. **SIMD / parallel reduction order.** Summing eight numbers as `((a+b) + (c+d)) + ((e+f) + (g+h))` differs from sequential summation by a few [ULP](glossary.md#ulp) (units in the last place — the gap between adjacent floating-point values). The harness offers a `deterministic` feature flag that forces sequential reduction in tests where this matters.
3. **[libm](glossary.md#libm).** `sin`, `exp`, `log`, `pow` differ by a few ULP across glibc, musl, Apple libm, and Windows ucrt (the system math libraries on each platform). Never recompute reference vectors in CI from libm calls; bake them into `tests/vectors/` once and load them with `npy::load_*`.

You only hit these when pushing tight tolerances (rtol ≤ 1e-13, SNR ≥ 140 dB). For the default thresholds in the table above, FMA contraction does not move the needle.

---

## Reference

Exhaustive description of every macro and helper. Each section is self-contained; jump in by name.

### assert_close!

```rust
assert_close!(actual, expected, rtol = R, atol = A);
```

Elementwise comparison with the mixed numpy / MATLAB / scipy tolerance: `|a − b| ≤ atol + rtol·|b|` at every index.

**Parameters**

- `actual` — array under test. Any type that dereferences to `&[f64]` or `&[f32]`.
- `expected` — reference array. Must have the same element type and length as `actual`.
- `rtol` — relative tolerance, a multiplier on `|expected|`. For a kernel that introduces some fraction of error proportional to the signal amplitude, this catches the deviation.
- `atol` — absolute tolerance, a floor for values near zero. Without it, zero-valued reference entries would require exact equality, which is rarely realistic after floating-point arithmetic.

Both `rtol` and `atol` are required by design. There is no useful default — the right values depend on the kernel.

**Use it for**: FIR / IIR output samples versus a precomputed reference; FFT bins versus a closed-form spectrum; resampler output; window taps versus scipy's window functions.

**Do not use it for**: stochastic outputs (use the SNR or KS assertions); FEC encoder output (use `assert_bit_exact!`); spectral magnitude validation (use `assert_spectral_mask!`).

**Failure diagnostic**: the panic message reports the first failing index, both values, the measured `|a − b|`, and the computed threshold. Length mismatches between `actual` and `expected` panic outright; they are usually a wiring bug in the test.

### assert_snr_db!

```rust
assert_snr_db!(actual, reference, min_db = MIN);
```

Computes the signal-to-noise ratio in dB, treating `reference` as the true signal and `actual − reference` as the noise. Passes when the SNR is at least `min_db`.

**Parameters**

- `actual` — array under test.
- `reference` — the reference signal. Typically the input to the kernel, or an analytic ground truth.
- `min_db` — minimum acceptable SNR in dB.

**Use it for**: filter output versus a noiseless input; FFT round-trip (forward then inverse) versus the original samples; polyphase resampler output; linear modulator output versus an analytic reference; AGC output during steady state.

**Do not use it for**: cases where the reference has zero energy (SNR is undefined; the assertion panics). For all-zero reference signals, use `assert_bit_exact!` instead.

**Notes**

- The SNR is computed in `f64` regardless of the sample type, so f32 and integer kernels can be tested without worrying about precision in the test itself.
- If the kernel's output is identical to the reference, the SNR is `+∞` and any finite `min_db` passes. The diagnostic still reports the threshold so you can see how much headroom the kernel had.

**Failure diagnostic**: measured SNR in dB, the threshold, and the constituent reference / error energies.

### assert_bit_exact!

```rust
assert_bit_exact!(actual, expected);
```

Element-by-element exact equality. The element type must implement `PartialEq` and `Debug`.

**Use it for**: kernels whose output representation is deterministic and required to match a spec exactly. FEC encoders (output bytes are defined by the standard); CRC outputs; scramblers; fixed-point arithmetic kernels (where the integer arithmetic is itself exact).

**Do not use it for**:

- **Floating-point kernels.** Float kernels are subject to platform-dependent rounding (FMA contraction, libm differences). Bit-exactness across platforms is not achievable in safe code.
- **Iterative soft-decision decoders** (turbo, LDPC, polar-SCL). Different implementations diverge on quantisation and scheduling; bit-exactness is the wrong correctness criterion. Use `assert_ber_at_ebn0!` instead.

**Failure diagnostic**: first failing index with both values printed via `Debug`.

### assert_spectral_mask!

```rust
assert_spectral_mask!(bins_db, lower = lower_db, upper = upper_db);
```

Tests that every bin in `bins_db` lies within `[lower_db[i], upper_db[i]]`. The bounds are per-bin, so the mask can be frequency-dependent — passband flat, transition steep, stopband per spec.

**Parameters**

- `bins_db` — slice of bin magnitudes in dB (one per frequency bin). The caller converts from linear magnitude to dB before passing in; the macro does not assume what your `0 dB` reference is.
- `lower_db` — slice of lower bounds in dB, same length as `bins_db`. Use `f64::NEG_INFINITY` for "no lower bound at this frequency."
- `upper_db` — slice of upper bounds in dB, same length. Use `f64::INFINITY` for "no upper bound at this frequency."

**Use it for**: validating filter designs against passband ripple / stopband attenuation specifications; transmit-spectrum compliance against regulatory masks (ETSI / FCC out-of-band emission limits).

**Do not use it for**: pointwise filter output sample comparison (use `assert_close!` or `assert_snr_db!`). The mask is about the *response shape*, not individual samples.

**Failure diagnostic**: first failing bin index, the measured dB level, and which bound was violated.

### assert_ber_at_ebn0!

```rust
assert_ber_at_ebn0!(
    errors,
    total_bits,
    target_ber = TARGET,
    ebn0_db = EBN0,
);
```

Asserts that the observed bit error rate (`errors / total_bits`) is at most `target_ber`. The `ebn0_db` parameter is the Eb/N0 the trial was run at — it does not affect the predicate, but it appears in the diagnostic so a failure is interpretable without consulting the test setup.

**Parameters**

- `errors` — number of bit errors observed (`u64`).
- `total_bits` — total bits transmitted in the trial (`u64`).
- `target_ber` — maximum acceptable BER.
- `ebn0_db` — Eb/N0 in dB at which the trial was run.

**Use it for**: FEC decoder validation against a published BER curve at known Eb/N0 points; demodulator slicer validation against closed-form AWGN BER formulas (BPSK / QPSK / 16-QAM / 64-QAM via the Q-function family).

**Do not use it for**: encoder validation (encoders are deterministic; use `assert_bit_exact!`).

**Conventional tolerances for iterative decoders**: **0.2 dB at the waterfall** (the steep portion of the BER curve where small Eb/N0 changes cause large BER changes) and **0.5 dB in the error floor** (where the curve flattens out at very low BER). Translate the Eb/N0 tolerance into a BER threshold at the operating point.

**Trial size matters**: a target BER of 10⁻⁶ needs on the order of 10⁸ bits transmitted to see ten errors and have a stable estimate. Tests that demand much lower BERs at small trial sizes are not useful.

**Failure diagnostic**: measured BER, target, and the Eb/N0 at which the trial was run.

### assert_parseval!

```rust
assert_parseval!(
    psd,
    bin_width_hz = DF,
    signal_energy = E,
    rtol = R,
);
```

Asserts that integrating the one-sided power spectral density (`Σ psd[k] · bin_width_hz`) recovers the time-domain signal energy (`(1/fs) · Σ |x|²`) within relative tolerance `rtol`.

**Parameters**

- `psd` — slice of PSD values (one-sided, units of power per Hz).
- `bin_width_hz` — the bin width in Hz (`fs / nfft` for unwindowed PSD; the caller is responsible for any window-correction factor).
- `signal_energy` — the reference signal's time-domain energy.
- `rtol` — relative tolerance on the integrated / reference ratio.

**Use it for**: spectral estimator validation (Welch, periodogram, multitaper). Different libraries scale their PSD differently; a kernel that gets the bin-width or window-correction factor wrong produces a PSD that looks plausible at first glance but does not integrate to the correct total energy.

**Do not use it for**: peak detection, frequency localisation, or spectral shape (use `assert_spectral_mask!` for shape). Parseval is about *total energy conservation*, not about whether the PSD points to the right frequency.

**Failure diagnostic**: integrated energy, reference energy, the ratio, and the tolerance.

### assert_distribution_ks!

```rust
assert_distribution_ks!(
    samples,
    cdf = TARGET_CDF_CLOSURE,
    alpha = ALPHA,
);
```

Runs the one-sample Kolmogorov-Smirnov test against `cdf` (a closure returning the target CDF's value at a point). Computes the D-statistic — the maximum vertical distance between the empirical and target CDFs — and compares against the critical value for the given significance level.

**Parameters**

- `samples` — slice of samples drawn from the implementation under test (`&[f64]`).
- `cdf` — a function `fn(f64) -> f64` returning the target CDF's value at a point.
- `alpha` — significance level. Supported values: 0.10, 0.05, 0.01, 0.001. Other values panic.

**Use it for**: PRNG output against a uniform distribution; AWGN generator output against a Gaussian distribution; similar "do these samples come from the distribution I claim they do" questions.

**Do not use it for**: dependencies between samples (autocorrelation, period structure). KS tests the marginal distribution only.

**Critical: fixed seed for CI, multi-seed for real statistical claims**

This macro is intended for CI use **with a fixed seed**. In that mode it acts as a deterministic vector regression — the assertion always passes (or always fails) for a given seed, with no statistical claim about the distribution.

To make a real statistical claim — *the implementation does produce samples from the target distribution* — you need many independent seeds, observing whether the p-values are uniform on `[0, 1]` under the null hypothesis. That kind of test belongs in a nightly or weekly job, not in PR CI: a true-positive rate of 1% at α = 0.01 means a one-in-100 chance of false failure per CI run, which is intolerable for a PR gate.

The harness does not enforce either pattern. It is on you to know which mode you're in:

- **Fixed-seed CI test**: deterministic vector regression. Set the seed at the top of the test, pick α once, get a flake-free test that catches regressions in this corner.
- **Nightly multi-seed validation**: run with many seeds, collect p-values, verify uniformity. The KS macro is one ingredient of that pipeline, not the whole pipeline.

**Failure diagnostic**: measured D-statistic, critical value, α, and sample size.

### honeyeater_test::npy

Loader for `.npy` reference-vector files committed under `tests/vectors/`.

```rust
use honeyeater_test::npy;
use std::path::Path;

let f32_data    : Vec<f32>             = npy::load_f32(Path::new("tests/vectors/x.npy"));
let f64_data    : Vec<f64>             = npy::load_f64(Path::new("tests/vectors/x.npy"));
let cf32_data   : Vec<Complex<f32>>    = npy::load_complex_f32(Path::new("tests/vectors/x.npy"));
let cf64_data   : Vec<Complex<f64>>    = npy::load_complex_f64(Path::new("tests/vectors/x.npy"));
```

numpy's complex format stores interleaved real / imaginary pairs, which is the same memory layout as a slice of `num_complex::Complex<T>`, so the loader returns the data in honeyeater's preferred type without a caller-side cast.

The loader supports 1-D arrays only. It refuses `.npy` files saved with `allow_pickle = True` as defence in depth.

Reference vectors are committed to the repository as opaque binary blobs. They are not regenerated in CI. The expectation is that whoever lands a kernel commits the reference vectors alongside it, with attribution to the oracle in a sibling text file.

For which oracles validate which kernels, see [Roadmap §Oracle stack](roadmap.md#oracle-stack-by-module-category-planned).

### honeyeater_test::scipy

Subprocess runner for live scipy cross-validation, for one-off checks where committing a `.npy` vector is overkill.

```rust
use honeyeater_test::scipy;

let json = scipy::run(r#"
import json
import numpy as np
from scipy.signal.windows import hann
print(json.dumps(hann(32).tolist()))
"#);
let reference: Vec<f64> = serde_json::from_str(&json).unwrap();
```

The runner requires Python with scipy and numpy installed locally. It is skipped (not failed) when no interpreter is available, so contributors without Python can still run the bulk of the suite.

**Pin your scipy version.** Different scipy releases have differed by a few ULP in higher-precision windows (notably Kaiser) and by more than that in elliptic IIR design at high order. The version pinning lives in `tools/oracle-gen/requirements.txt`.
