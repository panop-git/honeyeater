# honeyeater — roadmap

## Status

Pre-v0.0.1. Private development workspace. No code yet; no scaffolding files (no `Cargo.toml`, no CI). When a 0.0.1 release is ready, a new public repository will be created and the codebase migrated.

## The plan in one sentence

Stand up a small test harness with a principled [tolerance](glossary.md#tolerance-atol--rtol) vocabulary, then implement [Tier-1](glossary.md#tier-1) [kernels](glossary.md#kernel) test-first against named [oracles](glossary.md#oracle), with **CCSDS Reed-Solomon (255, 223)** as the first concrete standards-conformance demo that justifies cutting 0.0.1.

## Crate layout at 0.0.1

A Cargo workspace, deliberately small at first release:

```
honeyeater         facade crate
honeyeater-core    sample type, signal container, device traits, tolerance vocabulary
honeyeater-test    cross-validation helpers (dev-only, not shipped to users)
```

Additional crates split out when real dependency boundaries appear (a CUDA backend, a proc-macro, a heavy optional dep) — not pre-emptively. Reference points: `rustfft` and `realfft` are single-crate libraries (rustfft auto-detects AVX at runtime and exposes opt-in features for NEON and other paths); `tokio` and `ratatui` split crates only when a real boundary forces it.

A separate `tools/oracle-gen/` workspace, **outside the published crate set**, holds scripts that run libfec (LGPL) and other non-permissive oracles to produce binary test fixtures. The library's link graph never touches LGPL code. This pattern is used by `ring` for NIST CAVP vectors.

## Test methodology (planned)

Nothing in this section is implemented yet. It describes the intended shape of the test harness once it is built in Phase 0.

### Tolerance vocabulary

A small, fixed set of measures, intended to be used consistently across the codebase once implemented. Per-test thresholds will be documented per kernel; the set of measures themselves should be stable:

| Macro (planned) | Predicate | Intended for |
|---|---|---|
| `assert_close` | `\|a − b\| ≤ atol + rtol·\|b\|`, elementwise (numpy/MATLAB convention) | pointwise array comparison: FIR/IIR output, FFT bins, resampler output, window taps |
| `assert_snr_db` | `10·log10(Σ\|ref\|² / Σ\|ref − actual\|²) ≥ min_db` | structured signal-vs-reference: filters, FFT round-trips, resamplers, modulators, AGC |
| `assert_bit_exact` | exact equality at the byte (packed) or bit (unpacked) level, per the kernel's output representation | FEC encoders, fixed-point kernels, CRCs, scramblers |
| `assert_spectral_mask` | each bin within `[lower(f), upper(f)]` dB | filter design verification, transmit-spectrum compliance |
| `assert_ber_at_ebn0` | BER ≤ target at stated Eb/N0 over Monte-Carlo trials | FEC decoders, demodulator slicers |
| `assert_parseval` | one-sided PSD integral ≈ time-domain energy | spectral estimators (resolves the scipy/MATLAB/Octave Welch-scaling trap) |
| `assert_distribution_ks` | Kolmogorov-Smirnov test against target CDF | PRNGs, AWGN generators, noise sources |

Percentage (relative) tolerance is **not** the planned default. The field consensus (numpy, scipy, MATLAB, EBU, liquid-dsp) is the mixed predicate above for pointwise tests, because percentage breaks on zero crossings and is insensitive to dynamic range. Percentage survives only as an aggregate scalar metric (EVM, BER, loudness offsets) where it is genuinely appropriate.

### Default thresholds per module class (intended)

| Module class | Primary measure | Threshold | Source |
|---|---|---|---|
| FFT (f64) | SNR | ≥ 120 dB | comfortably loose vs. FFTW's typical `O(log N · ε)` ≈ 280 dB at N=2²⁰; Higham §24 |
| FFT (f32) | SNR | ≥ 60 dB | RustFFT, scipy convention |
| FIR output | SNR | ≥ 100 dB (f64), ≥ 60 dB (f32) | liquid-dsp practice |
| FIR design | spectral mask | passband ±0.1 dB, stopband per spec | scipy `remez` tests |
| IIR | SNR vs scipy `lfilter` | ≥ 80 dB (f64) | scipy.signal tests |
| Polyphase resampler | SNR | ≥ 80 dB | liquid-dsp `resamp_crcf` |
| Window functions | mixed | rtol=1e-12, atol=1e-15 | scipy.signal.windows tests |
| Linear modulator | SNR vs analytic reference | ≥ 100 dB (f64) | liquid autotest convention |
| FEC encoder | bit-exact | byte equality | G.191 / CCSDS regime |
| FEC decoder (iterative) | BER at Eb/N0 | spec-dependent | DVB-S2, 3GPP, CCSDS |
| EVM aggregate | percent | per 3GPP TS 36.104 | 3GPP |
| AWGN / PRNG | KS + moment match | KS α=0.01 (with fixed seed in CI), mean/var within 3σ | numpy practice |
| AGC | SNR + settling | within ±0.5 dB steady-state | liquid `agc_crcf_autotest` |

### Cross-platform reproducibility

Floating-point reproducibility across x86 / aarch64 / glibc / musl / Apple libm is not free. Mitigations to bake into the test harness when it is built:

- Force `-ffp-contract=off` in test config (or pin a no-FMA reference path) so FMA contraction doesn't change results across ISAs.
- Offer a "deterministic" feature flag that forces sequential reduction in tests, so SIMD/parallel reduction order doesn't change FFT/dot-product results across CPUs.
- Bake reference vectors into `tests/vectors/` as `.npy` files — never recompute libm references in CI, since `sin`/`exp`/`log` differ by a few ULP across libm implementations.
- Force IEEE compliance in test config (no subnormal flush-to-zero).
- Pin oracle versions: `requirements.txt` next to fixture-generation scripts should record exact scipy / numpy versions so vectors are reproducible.

### Statistical tests in CI

Tests that *look* statistical but run in PR CI should use a fixed seed and act as deterministic vector regressions — flake-free, no real distributional claim. The actual statistical question (does the implementation match the target distribution?) belongs in a separate nightly / weekly job that runs across many independent seeds and checks that the resulting p-values are uniform on [0,1] under the null. This only works cleanly for tests with continuous null distributions (KS, Anderson-Darling); chi-square's discrete bins distort uniformity at small N and need a coarser pass/fail criterion.

## Oracle stack by module category (planned)

The library's content is intended to be organised into six categories. Each has a primary numerical oracle (and a secondary cross-check where one is genuinely independent). None of these oracles are wired up yet:

### Cat 0 — Numerical kernels (prerequisite layer)
Matrix decompositions, polynomial roots, special functions, sequence generators (Gold, Kasami, m-sequences, Zadoff-Chu, Barker, PN).

- **Primary oracle:** scipy.linalg + Boost.Math
- **Tested first**, before anything that depends on it.

### Cat 1 — Transforms and spectral decomposition
FFT, DCT, STFT, Hilbert, wavelet (deferred), spectral-estimation algorithms (Welch, periodogram, multitaper).

- **Primary oracle:** scipy.signal + scipy.fft (BSD-3)
- **Secondary:** FFTW via `pyfftw`, because scipy uses pocketfft and FFTW is genuinely independent
- **Known weaknesses:** scipy's high-order elliptic and `firls` have documented divergences from MATLAB; Octave's `signal` package is the tiebreaker for those corner cases. Always generate filter coefficients in SOS form when comparing.
- **FFT delegation (planned):** rustfft will be the implementation. honeyeater will wrap it for API consistency rather than reimplement.

### Cat 2 — Filters and resampling
FIR/IIR/biquad design, polyphase, adaptive filters in their *filter* role (LMS as denoiser), integer/rational/Farrow resamplers.

- **Primary oracle:** scipy.signal for design coefficients and execution
- Tests RF resampler quality on aliasing rejection / SNR in dB, not perceptual metrics.

### Cat 3 — Modulation, synchronisation, framing, equalisation
PSK/QAM/FSK/CPM mod/demod, OFDM (standards-conformant only, deferred to Tier 2 work), Costas/PLL/Gardner/M&M, adaptive filters in their *equaliser* role, frame sync.

- **Primary oracle:** liquid-dsp (MIT), vendored at a pinned commit, bindgen regenerated against current toolchain. The `liquid-dsp-bindings-sys` crate on crates.io is stale (2019) — do not depend on it; regenerate.
- **Secondary:** GNU Radio QA test patterns via subprocess for sync-loop convergence (GPL-3, so subprocess-only, never linked).
- **Critical gap to fill ourselves:** liquid-dsp's modem autotests are noiseless round-trip equality checks, not BER curves. honeyeater will need to supply its own closed-form AWGN BER assertions (BPSK/QPSK/16-QAM/64-QAM via the standard `Q(sqrt(2·Eb/N0))`-family formulas). This is expected to be the single most important piece of test infrastructure for Cat 3.
- **Standards-conformant OFDM** (LTE / 5G NR / DVB-T2 / DVB-S2X): MATLAB-captured vectors, shipped as opaque test data with attribution. liquid's own `ofdmflexframe` is a non-standard hand-rolled waveform — useful as a self-consistency oracle, not a conformance one.

### Cat 4 — Forward error correction
CRC, Hamming, Golay, Reed-Solomon, convolutional + Viterbi, BCH, LDPC.

- **Per code-family oracle:**
  - **CRC**: reveng catalogue (public) — constants compiled in, no library dependency, `"123456789"` standard check value
  - **Hamming / Golay**: textbook generator matrices, exhaustive enumeration of small codes
  - **Reed-Solomon (CCSDS RS(255, 223))**: KA9Q libfec (LGPL) — vectors captured ahead-of-time in `tools/oracle-gen/`, shipped as opaque binary blobs, no link dependency. Normative cross-check against CCSDS 131.0-B-5 §4 (which fixes the code parameters, generator polynomial, and dual-basis representation) plus JPL TMOD 810-005 module 208 for worked numerical examples. CCSDS 131.0-B-5 Annex F documents the Berlekamp↔conventional basis transformation needed when comparing against any oracle that operates in the conventional basis.
  - **Convolutional K=7 r=1/2 (CCSDS / Voyager standard)**: libfec for encoder bit-exactness; BER curve for decoder
  - **BCH (DVB-S2 outer)**: AFF3CT (MIT) and ETSI EN 302 307-1 §5.1.1 polynomial
  - **LDPC (DVB-S2, CCSDS AR4JA)**: AFF3CT for encoder vectors; BER curves vs ETSI TR 102 376-1 and published CCSDS plots
- **Decoders generally**: bit-exact testing is impossible for iterative soft-decision decoders (turbo, LDPC, SCL polar) because implementations diverge on quantisation and scheduling. BER vs Eb/N0 is the only sensible test, with 0.2 dB tolerance at waterfall and 0.5 dB in the error floor.
- **AFF3CT** (MIT) is the intended workhorse oracle for everything iterative; it ships reference BER curves that honeyeater would compare against.
- 5G NR Polar codes and LTE turbo codes are not Tier 1 (limited deployment outside cellular infrastructure) — defer.

### Cat 5 — Stochastic sources and channel models
PRNGs, AWGN, Rayleigh/Rician/Nakagami fading, 3GPP TDL, statistical properties of estimators.

- **PRNG strategy:** delegate to the Rust `rand` and `rand_distr` crates rather than reimplementing. One-time qualification report via TestU01 BigCrush and PractRand to 1 TB, archived in `docs/prng-qualification.md`. ChaCha and PCG both pass; the report is a formality but a citable one.
- **AWGN and distributions:** scipy.stats for CDF/moment cross-checks. Anderson-Darling on large sample sizes (N ≥ 10⁶) to catch tail-handling bugs (Ziggurat has historical edge-case bugs in the tail).
- **Fading channels:** 3GPP TR 38.901 §7.7.2 (TDL profiles A–E, Tables 7.7.2-1 through 7.7.2-5) and §7.7.1 (CDL profiles A–E) as the spec targets. No bit-exact reference exists for these — test marginal envelope (KS against Rayleigh/Rice), Doppler PSD shape against Clarke/Jakes analytical, tap PDP against the relevant table within ±0.1 dB.

## Implementation order

### Phase 0 — Scaffolding

Before any kernel is implemented:

1. Cargo workspace skeleton: workspace `Cargo.toml` at root, `honeyeater` (facade), `honeyeater-core` (sample types, trait definitions, signal containers), `honeyeater-test` (cross-validation helpers).
2. The `Sample` trait plus the fixed-point sample type set in `honeyeater-core`: `Complex<i16>` and `Complex<i8>` as kernel sample types (`Sample`-implementing); `Complex<u8>` as a transport-only type at the SDR boundary, debiased to one of the others before any kernel touches it (the three integer formats produced by SDR hardware across the field — see `docs/architecture-planning.md` for the landscape and decisions 5–6 for the rationale). The trait is satisfied by `f32`, `f64`, `i16`, `i8` (and their `Complex<…>` wrappings). Without this wiring, generic kernels can't be written and fixed-point can't ship at 0.0.1. Goes in before any kernel.
3. Wire up the `rustfft` dependency in `honeyeater-core` (used by Phase 1 step 4, the FFT wrapper). `num-complex` re-exported through `honeyeater-core` so user code has a stable import path independent of rustfft's version pinning.
4. `honeyeater-test`: the seven assertion macros, a `.npy` loader for committed reference vectors, a scipy-subprocess helper for live cross-validation. **This is the highest-leverage piece of infrastructure in the project.**
5. The `tools/oracle-gen/` workspace, outside the published crate set, for generating reference vectors from libfec / AFF3CT / etc. without those libraries entering the library's link graph.
6. CI: a single `ci.yml` with `cargo fmt --check`, `cargo clippy -D warnings`, `cargo test`, `cargo doc` with `RUSTDOCFLAGS=-Dwarnings`, plus `cargo deny check`. Stable + MSRV + nightly.
7. Repo hygiene files: `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md` (a short house-written conduct statement — originally "copy Rust's", revised: the Rust CoC and the Contributor Covenant both promise a staffed moderation and report-handling process that a company-maintained project of this size does not offer; the policy promises only what the maintainers actually do, which is discretionary curation with GitHub's standard tools), `SECURITY.md`, `rustfmt.toml`, `clippy.toml`, `deny.toml`.
8. Reserve `honeyeater`, `honeyeater-core`, `honeyeater-test` on crates.io as `0.0.1-alpha.0`.

### Phase 1 — Tier-1 RF/electrical primitives, in order

In rough order of "fastest validation win × highest user value":

1. **Hann window** — trivial, scipy bit-near oracle, exercises the entire test harness end-to-end before anything risky is built. Float-only initially (windows aren't typically fixed-point).
2. **Hamming, Blackman-Harris, Kaiser windows** — same harness, rounds out the window family.
3. **RBJ biquad coefficients + execution** — first filter, validates the design-coefficient testing path (formulas are the spec; cross-check execution against scipy `sosfilt`).
4. **FFT wrapper** delegating to rustfft (complex-in / complex-out) — establishes the signal-type plumbing. Float-only (FFT in fixed-point is a separate non-trivial implementation). Real-input FFT (real → conjugate-symmetric complex, the `realfft` shape) is not part of 0.0.1; add when a kernel needs it.
5. **CRC-32 (Castagnoli) and CRC-16** — first bit-exact test, reveng oracle, tiny code, no external dep.
6. **NCO / DDS** — first stateful kernel, exercises SFDR property testing. **Implemented in both float and fixed-point** (`Complex<i16>` and `Complex<i8>`) — this is the first fixed-point kernel; it's small and well-defined so it's a good first proof of the trait machinery.
7. **SDR sample boundary helpers** — conversions between `Complex<i16>` / `Complex<i8>` (kernel sample types) and `Complex<f32>` / `Complex<f64>`, with Q-format scaling as a parameter (so the same `i16` conversion serves USRP `sc16` at Q1.15 and BladeRF `SC16_Q11` at Q1.11 by passing the right scale), plus `Complex<u8>` → `Complex<i8>` / `Complex<f32>` debiasing for RTL-SDR (subtracting the 127.5 midpoint), plus optional deinterleave to separate I/Q arrays. Trivial code, but unblocks every SDR user. Tested by round-trip identity (for the lossless paths), value-range checks, and bias-handling correctness for the RTL-SDR path. **Ships alongside** a `q_format` module of named per-radio constants covering every radio with a current SoapySDR support module (USRP, BladeRF including `SC16_Q11`, `SC16_Q11_PACKED`, and `SC8_Q7` modes, HackRF, RTL-SDR, Airspy R2/Mini, Airspy HF+, SDRplay, Pluto with separate RX/TX constants, LimeSDR including the `CS12` packed variant, FCDPP, Sidekiq, Mirics, Red Pitaya, XTRX, Iris, NetSDR/Afedri, plus the SoapyOsmo/SoapyAudio/SoapyRemote shims) so users pass the constant for the radio they own rather than typing a raw scale. See `docs/architecture-planning.md` decision 6 for the full table.
8. **FIR filter execution** — **implemented in both float and fixed-point.** Hot-path kernel; native fixed-point is what makes high-rate streaming receivers viable across SDR vendors. Cross-validate float version against scipy `lfilter`; cross-validate fixed-point against the float version (within Q-format quantisation bounds).
9. **Complex multiply** (the mixer primitive) — **implemented in both float and fixed-point.** Tiny but ubiquitous; needed alongside the NCO for downconversion.
10. **CCSDS Reed-Solomon (255, 223) encoder** — **first standards-conformance demo.** Bit-exact against CCSDS 131.0-B-5 Annex F worked examples plus libfec-generated vectors. This is the milestone that justifies cutting 0.0.1.

Tier 1 then continues (no fixed-point unless explicitly noted): FIR design (window method, Parks-McClellan), IIR design (Butterworth/Chebyshev/Elliptic), polyphase resampling (consider fixed-point), mixer / IQ imbalance / DC offset removal (consider fixed-point), AGC (consider fixed-point), AWGN channel, PLL / Costas loop, Mueller & Müller timing recovery, linear modems (BPSK / QPSK / 8PSK / 16-QAM / 64-QAM), CPFSK / GMSK, Viterbi decoder, LMS / RLS equaliser. None of these block 0.0.1 — they ship as they're ready.

### Phase 2 — Tier-2 specialty (post-0.0.1)

3GPP TDL channel models, ITU-R IMT channel models, standards-conformant OFDM (LTE / 5G NR / DVB-T2 / DVB-S2X) with MATLAB-captured vectors, DVB-S2 LDPC, CCSDS LDPC (AR4JA family), BCH (DVB-S2 outer), spectral-estimation rigour (multitaper, Lomb-Scargle), EVM / MER / SNR estimators, spurious / SFDR / phase-noise measurement primitives.

### Deferred indefinitely

- Wavelet transforms — no demand signal for an RF/electrical library
- 5G NR Polar codes — niche outside cellular infrastructure
- LTE turbo codes — superseded by LDPC in new designs
- LDPC for Wi-Fi specifically — Wi-Fi isn't a typical defence/space target
- ISDB / ATSC FEC variants
- Audio-perceptual kernels (entire category — out of scope; existing Rust library covers it)
- High-assurance certification tooling (interval arithmetic, formal harness, requirements-traceability infrastructure) — premature without a target certification authority locked in

## Cutting 0.0.1

When the milestone in Phase 1 step 10 is achieved (CCSDS RS(255, 223) bit-exact), and the harness, CI, scaffolding files, and the minimal kernel set are all green, cut 0.0.1 to a fresh public repository. The minimum kernel set for 0.0.1 is:

- Window family (Hann, Hamming, Blackman-Harris, Kaiser)
- RBJ biquad design and execution (float)
- FFT wrapper around rustfft, complex-in / complex-out (float)
- CRC-32 and CRC-16
- NCO / DDS (float **and** fixed-point)
- SDR sample boundary helpers (integer↔float, Q-format-aware)
- FIR filter execution (float **and** fixed-point)
- Complex multiply (float **and** fixed-point)
- CCSDS RS(255, 223) encoder (bit-exact against Blue Book vectors)

The current working tree remains the private development workspace; the public repo gets the polished cut.
