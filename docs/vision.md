# honeyeater — vision

## What it is

A Rust library of digital signal processing primitives for radio-frequency and electrical signals, intended for host-side processing of sample streams from radios, digitisers, and simulators. CPU-first today, with GPU acceleration designed for as a feature-flagged long-term addition (see [Long-term](#long-term-deferred-but-designed-for)).

## What it is for

The target user is someone building software that takes a stream of [IQ samples](glossary.md#iq-samples) (the pairs of numbers a radio hands to software, describing the signal moment by moment) produced by a radio front-end, a digitiser, or a simulation, and does something useful with it on a host. Concretely:

- Satellite ground-station software
- Telemetry decoders
- Spectrum-analyser back-ends
- Signal-intelligence processing chains
- Modem implementations on workstation, server, embedded Linux, and microcontroller targets
- Bench-test and instrumentation tooling

## What it is not

- Not FPGA or ASIC gateware. The high-rate sample-by-sample work that runs inside the radio itself is a different toolchain (VHDL/Verilog/HLS) with different verification methodology. honeyeater starts where the [ADC](glossary.md#adc)'s samples (the digitised signal, straight off the analog-to-digital converter) reach the host.
- Not an audio-perceptual library. Loudness, dynamics, reverb, codec primitives, and pitch detection are served by existing Rust libraries. General-purpose primitives (filters, transforms, resampling) work on audio-rate signals, but the design priorities are RF and electrical.
- Not a flowgraph runtime (GNU Radio's role). Not a radio HAL (SoapySDR's role). honeyeater is a library of primitives.

## Exclusion on principle

Civilian, general-purpose. Waveforms defined only in restricted military standards (LPI/LPD, anti-jam, milspec SATCOM, tactical-link physical layers) and anything controlled under US ITAR or restricted EAR classifications are excluded by policy, regardless of contributor enthusiasm. General primitives used in defence systems — FFT, Reed-Solomon, LDPC, PLLs — are in scope as dual-use civilian-standard technology used openly across satcom, broadcast, and instrumentation.

## Design principles

1. **Test-driven from the start.** Every kernel is validated against a named [oracle](glossary.md#oracle) (a trusted reference that says what the right answer is): an established reference implementation (scipy.signal, liquid-dsp, libfec, AFF3CT) or bit-exact vectors from a published standard (CCSDS Blue Books, ETSI, 3GPP). Numerical [tolerance](glossary.md#tolerance-atol--rtol) — how close counts as correct — is a first-class part of every public API.
2. **A small, principled tolerance vocabulary.** The codebase uses one consistent set of tolerance measures (elementwise `atol + rtol·|b|` close-comparison, SNR in dB, bit-exact, spectral mask, [BER](glossary.md#ber-bit-error-rate) at [Eb/N0](glossary.md#ebn0), [Parseval](glossary.md#parsevals-theorem) energy, [Kolmogorov-Smirnov](glossary.md#kolmogorov-smirnov-ks-test) distribution test) rather than ad-hoc thresholds scattered through test files. Each measure is defined, with worked examples, on the [Testing](testing.md) page.
3. **Memory-safe by construction.** Written in Rust. The class of bugs that dominate C DSP libraries (buffer overruns, use-after-free, data races) is structurally excluded for safe code.
4. **Generic over sample type, including fixed-point.** Algorithms are written once over a `Sample` trait satisfied by `f32`, `f64`, and the signed fixed-point sample types common to SDR hardware (`i16` and `i8` with [Q-format](glossary.md#q-format-fixed-point) scaling — a way of storing fractional values inside plain integers) — and [monomorphised](glossary.md#monomorphisation) per concrete type by the compiler (specialised into a dedicated copy for each type, so the generality costs nothing at runtime). This serves both the high-precision case (f64 for filter design and high-dynamic-range work) and the high-rate case (native fixed-point for streaming receivers where the boundary conversion to float is prohibitive at full rate). The RTL-SDR's biased `Complex<u8>` is supported via boundary debiasing helpers rather than as a kernel sample type — see `docs/architecture-planning.md` decisions 5 and 6.
5. **Plain buffers, no metadata wrapper.** [Hot-path](glossary.md#hot-path) APIs — the per-sample code that runs most often — take `&[T]` and `&mut [T]`. Sample-rate metadata is the runtime's or the application's concern, not the library's — matching what every other pure DSP library (rustfft, scipy.signal, liquid-dsp) does.
6. **Pay only for what you use.** A Cargo workspace of small crates, split when real dependency boundaries appear, not pre-emptively.

## Comparators

honeyeater sits in a landscape dominated by:

- **[liquid-dsp](https://liquidsdr.org)** — C, permissive licence, single-maintainer, RF-focused. The primary numerical reference for modulation and synchronisation. Macro-driven type families. honeyeater's most direct comparator.
- **[GNU Radio](https://www.gnuradio.org)** — C++ flowgraph runtime with foundation governance. Different shape of project (runtime + block library). GPL-3.
- **[scipy.signal](https://docs.scipy.org/doc/scipy/reference/signal.html)** — Python, BSD-3, the de facto reference for filter and spectral algorithms in scientific computing. honeyeater's primary numerical reference for Tier 1 kernels.
- **[rustfft](https://github.com/ejmahler/RustFFT), [realfft](https://github.com/HEnquist/realfft), [futuresdr](https://www.futuresdr.org), [rustradio](https://github.com/spectralfuture/rustradio)** — existing Rust DSP work, narrower in scope. honeyeater does not duplicate rustfft; it depends on it. realfft (real-input variant by the same family of authors) is a likely future addition but not part of 0.0.1.

The whitespace in Rust is large: IIR design pipeline, full window family, [Parks-McClellan](glossary.md#parks-mcclellan), modems, [FEC](glossary.md#fec-forward-error-correction) beyond toy CRC, [OFDM](glossary.md#ofdm), symbol/carrier sync, channel models, and electrical-instrumentation primitives are all sparsely covered or absent.

## Long-term (deferred, but designed-for)

- **GPU backend** — CUDA first. Not part of 0.0.1. When added, will live behind a feature flag in a separate crate (`honeyeater-cuda`) so the core library has no GPU dependencies.
- **`no_std` support** — for embedded targets (Cortex-M, RISC-V microcontrollers). Not part of 0.0.1. Kernel signatures designed to avoid gratuitous heap allocation so this remains tractable later.
- **Formal certification posture** — IEC 61508, DO-178C, ISO 26262 are heavyweight regimes that require concrete evidence (requirements traceability, MISRA-equivalent lints, documented worst-case error bounds). Long-term aspiration only. No claims made until concrete evidence exists.
