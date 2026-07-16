# Architecture planning

Architectural decisions for honeyeater pre-0.0.1, with rationale. All decisions below (numbered 1–10, with 3a as a sub-decision under 3) are now resolved. This document is the authoritative reference for design intent; if a future contributor wants to deviate from one of these, the deviation needs to update this file with reasoning.

## Constraints

### Downstream use: the Panop flowgraph

honeyeater is open-source and general-purpose, but it has a known downstream consumer: an internal Panop flowgraph runtime, GNU Radio-style, used for RF DSP. honeyeater stays a library of primitives — the runtime is a separate project, not part of honeyeater.

Two properties of the Panop runtime constrain honeyeater's design:

1. **The flowgraph is built at compile time.** Different waveforms (AIS receiver, DVB-S2 receiver, etc.) are pre-compiled into separate graph artefacts. Hot-swap of waveforms is "stop running graph A, start running graph B," not "reconfigure the existing graph in place." Each graph's blocks have their parameters known at compile time.
2. **Multiple coexisting RX/TX chains.** The runtime can run several independent graphs at different sample rates simultaneously. honeyeater must not assume there's one global sample rate, one global processor configuration, or one graph in flight.

This rules out a JUCE-style `prepare(spec)` step separated from construction, because the spec is known at compile time and there's no separate runtime configuration step. (JUCE is a widely used C++ audio framework; its processors take a one-time `prepare()` call carrying the sample rate and block size before processing begins — a pattern that suits a single audio device but not honeyeater's many independent RF chains.) It matches liquid-dsp's pattern: parameters baked in at construction, processor immutable for its lifetime, sample rate handled in normalised form (cycles per sample) wherever possible.

### Hardware: SDR sample-format landscape

honeyeater is a general-purpose RF DSP library and must be cheap to use with the full range of SDR hardware in the field. The realistic intake set, ordered by frequency of occurrence (verified against SDK sources):

- **Interleaved `Complex<i16>`** — the dominant native wire format. USRP (UHD `sc16`, full-scale ±1.0 ↔ ±32767, i.e. Q1.15), BladeRF (`SC16_Q11`: signed 16-bit container, 11 fractional bits, full-scale ±1.0 ↔ ±2048, leaving 4 unused MSBs of sign-extension headroom), LimeSDR (`I16`, 12-bit LMS7002M MSB-aligned in i16, Q1.15), Epiq Sidekiq, ADALM-Pluto (12-bit AD9361 sample — RX Q1.11 LSB-aligned, TX Q1.15 MSB-aligned), Airspy R2/Mini (12-bit MSB-aligned, Q1.15), SDRplay (14-bit MSB-aligned, Q1.15), SoapySDR `CS16`. Almost every recent SDR delivers this by default.
- **Interleaved `Complex<i8>`** — high-rate / oversample mode. BladeRF (`SC8_Q7`, 122.88 MS/s), USRP `sc8`, HackRF One (only format).
- **Interleaved `Complex<u8>` with DC midpoint at 127.5** — RTL-SDR only. The hardware delivers unsigned-8 samples in `[0, 255]`, with the true zero-signal value sitting at 127.5 (the midpoint of the range); processing code conventionally subtracts either 127 or 128 depending on whether it's targeting `i8` arithmetic or preserving the exact midpoint. Common in entry-level and hobbyist captures.
- **Interleaved `Complex<f32>`** — the host-side lingua franca after conversion. SoapySDR `CF32`, UHD `fc32`, SigMF default for capture files.
- **Interleaved `Complex<f64>`** — rare; UHD `fc64` and Soapy `CF64` only. Acceptable to handle by conversion rather than zero-copy.

The various 12-bit-in-int16 sub-variants (LimeSuite `I12`, Pluto, Sidekiq) are bit-identical to the generic interleaved `Complex<i16>` from a memory-layout standpoint; only the Q-format scale factor differs. So they are handled by the same memory layout plus a scaling constant.

This means honeyeater must support **native fixed-point processing** on its hot-path kernels, not just floating-point with boundary conversion:

- At 122.88 MS/s on one core (the BladeRF oversample case), each sample has ~8 ns of compute budget. Conversion to f32 at the boundary eats meaningfully into that. Native int8 SIMD on AVX2 has roughly 4× the per-cycle throughput of f32 SIMD.
- The same argument applies in milder form to any high-rate streaming receiver on any radio in the list.
- A library that requires float at the boundary effectively limits its users to perhaps 30–60 MS/s on a modern CPU — well below what BladeRF, USRP X310, LimeSDR, and Sidekiq X4 can deliver.

Library design implication: ship `Complex<i16>`, `Complex<i8>`, `Complex<u8>` as first-class hot-path sample types alongside `Complex<f32>` and `Complex<f64>`, with the `Sample` trait satisfied by all of them.

(Sources: [SoapySDR Formats.h](https://raw.githubusercontent.com/pothosware/SoapySDR/master/include/SoapySDR/Formats.h), [UHD configuration](https://files.ettus.com/manual/page_configuration.html), [Sidekiq SDK manual](https://resources.epiqsolutions.com/hubfs/Sidekiq_Software_Development_Manual.pdf), [hackrf.h](https://github.com/mossmann/hackrf/blob/master/host/libhackrf/src/hackrf.h), [LimeSuite.h](https://raw.githubusercontent.com/myriadrf/LimeSuite/master/src/lime/LimeSuite.h), [rtl-sdr.h](https://github.com/osmocom/rtl-sdr/blob/master/include/rtl-sdr.h), [libiio AD9361 example](https://github.com/analogdevicesinc/libiio/blob/main/examples/ad9361-iiostream.c), [libbladeRF.h](https://github.com/Nuand/bladeRF/blob/master/host/libraries/libbladeRF/include/libbladeRF.h).)

### Reference libraries to look to

When a design question arises, look to **liquid-dsp** (RF, permissive, single-maintainer but the closest peer) and **scipy.signal** (BSD, the de facto reference for filter and spectral algorithms) first. JUCE and other audio-DSP libraries are not appropriate references because their design centres a single-rate single-processor "plugin in a DAW" model that doesn't match the multi-RX/TX-chain RF case. Flowgraph SDR frameworks (GNU Radio, FutureSDR) are useful for understanding stream-metadata patterns but not for honeyeater's library shape, since honeyeater isn't a runtime.

Wherever a claim is made about how another library behaves, verify it. The ecosystem-claim about JUCE carrying sample rate on its buffer was wrong on inspection; sample-rate metadata in JUCE lives only in a `ProcessSpec` passed once to `prepare()`. Verify by reading source / docs, not from memory.

## Decisions

### 1. MSRV — latest stable minus two

The Rust compiler version honeyeater promises to compile under is the version released about three months before the most recent stable (i.e. N-2 across Rust's six-week release cadence). This is a common pattern in production-leaning crates; the ecosystem has no unified MSRV policy (tokio uses roughly N-4, serde leaves it open, others vary), so we pick the spot that balances toolchain lag in regulated environments against not freezing on genuinely old compilers.

### 2. Sample-data ownership — borrowed-output and stateful processors primary; owned-output as documented convenience

The hot path uses two complementary forms:

- **Borrowed output** — `fn process(input: &[T], output: &mut [T])`. Caller pre-allocates both buffers. Zero allocations per call. Primary form for stateless operations (windows, gain adjustment, fixed-point↔float conversion).
- **Stateful processors** — objects constructed once, holding internal state, called repeatedly. Primary form for anything carrying state between calls (filters, NCOs, PLLs, demodulators, AGCs, equalisers).

An **owned-output** form (`fn process(input) -> Vec<T>`) is offered as a thin convenience layer for offline analysis, filter design, one-shot processing. It is a three-line wrapper on top of the borrowed-output primitive, not a separate implementation.

Documentation for the owned-output form must include words to this effect: *"Allocates a fresh output buffer on every call. Suitable for offline analysis, filter design, one-shot processing, and any use where occasional unpredictable delays are acceptable. Not suitable for hard real-time streaming pipelines (e.g. live SDR receive chains where a missed buffer drops samples)."*

Naming convention for the owned variants to be settled at first-kernel time (likely `_owned` suffix or a `convenience` submodule).

### 3. Signal type — plain buffers, no metadata wrapper

honeyeater uses plain Rust slices everywhere on the hot path. There is no `Signal<T>` struct carrying sample rate; sample rate is either a normal constructor argument to a processor (when needed) or handled in normalised form (cycles per sample) so the processor doesn't need to know the absolute rate.

This matches what every pure DSP library does: rustfft, ndarray, nalgebra, scipy arrays, and liquid-dsp all use plain buffers. Where stream metadata exists (GNU Radio's stream tags, scipy's `dlti.dt`), it lives on the *stream* or the *system object* — not on the sample buffer. honeyeater isn't a runtime, so stream-tag machinery doesn't belong in it; the Panop flowgraph runtime will provide that.

Sample-rate metadata that genuinely needs to flow with data (e.g. for a spectrum-analyser GUI displaying frequencies in Hz) is the *application's* responsibility, not the library's. honeyeater provides bin spacing in normalised form and a helper to convert to Hz given fs.

### 3a. Block parameter mutability — per-block, mutators only where safe

A flowgraph block can have its parameters fixed in three ways:

1. **Compile-time constants** the compiler folds into the hot path (Panop's typical case for waveform-defining parameters).
2. **Construction-time values** stored in the struct (the default for everything else).
3. **Runtime-mutable parameters** with setter methods.

honeyeater's policy: every processor takes its parameters at construction. Mutator methods are exposed only on processors where in-place mutation is **safe** — meaning the change applies cleanly to future samples without producing a transient and doesn't require re-deriving derived state. Examples:

- **Exposes setters**: `Nco::set_frequency`, `Agc::set_reference`, `Gain::set_db`.
- **No setters; reconstruct to change**: `FirFilter` (coefficient change causes delay-line transient), `IirFilter` (same), `RrcShaper` (precomputed coefficient tables).

Graph-level hot-swap (stop graph A, start graph B) is independent of per-block mutability and works regardless of which form a block uses.

### 4. Complex-number representation — `num_complex::Complex<T>`

[IQ samples](glossary.md#iq-samples) use `num_complex::Complex<T>` from the `num-complex` crate. This is a struct of two `T` values (real and imaginary parts) laid out adjacent in memory, so an array of `Complex<f32>` has the same byte layout as interleaved `[I0, Q0, I1, Q1, ...]` arrays. This is the layout every SDR driver in the field produces — zero-copy at the hardware boundary regardless of which radio supplies the samples.

`num_complex` is what `rustfft` uses, so honeyeater's FFT wrapper has the same type at its interface. Compile-time real-vs-complex distinction is free: a function written for `&[f32]` cannot be called with `&[Complex<f32>]` and vice versa, ruling out a common class of bug in C SDR code.

For fixed-point IQ, the same wrapping applies: `Complex<i16>` and `Complex<i8>` are valid sample types with the same struct layout and the same compile-time real/complex discipline. `Complex<u8>` (RTL-SDR's biased format) uses the same struct layout but is treated as a transport-only type at the SDR boundary, not a kernel sample type — see decision 6. Q-format scaling (Q1.15, Q1.11, Q1.7, etc.) is a property of *how the sample is interpreted by surrounding code*, not a property of the type itself.

### 5. Generic over sample type — `T: Sample`

Every kernel that can be is written once, generic over a placeholder sample type `T`, with a trait bound `T: Sample`. The Rust compiler stamps out specialised versions per concrete type ([monomorphisation](glossary.md#monomorphisation)) — zero runtime cost compared to hand-written duplicates.

`T: Sample` is a honeyeater-defined trait, **not** `num_traits::Float` and deliberately not named `Real` (which would collide with `num_traits::real::Real`, the existing real-number trait in that crate). (`num_traits` is the de-facto Rust crate of numeric abstraction traits; reusing one of its trait names for a different concept would confuse anyone who already knows it.) The distinction matters: `Float` excludes fixed-point types by definition (it requires NaN, infinity, etc.), and decision 6 below commits honeyeater to fixed-point sample types. `Sample` is a strictly smaller trait — addition, multiplication, negation, comparison, conversion to/from a few standard scalar types — satisfied by `f32`, `f64`, the signed integer sample types `i16` and `i8` (with sign-symmetric arithmetic that matches signal semantics), and anything else useful future contributors define. `Complex<u8>` is *not* a `Sample` type; see decision 6.

The trait bound design is load-bearing for the fixed-point story; getting it right at the start saves a breaking change later. The name `Sample` is the universal term in DSP for "one element of a signal stream" and avoids the readability tax of a name clash with `num_traits`.

### 6. Fixed-point support — float and fixed-point from day one

Native fixed-point sample types ship at 0.0.1, alongside `f32` and `f64`. The rationale:

- Every major SDR family (USRP, BladeRF, LimeSDR, Sidekiq, HackRF, RTL-SDR, Pluto) delivers fixed-point samples natively. f32 is a host-side convenience after conversion; the hardware speaks integer.
- At streaming-receiver rates above roughly 30–60 MS/s on a modern CPU core, the boundary conversion from integer to f32 begins to eat a meaningful slice of the per-sample budget. By 122.88 MS/s (the high end currently in use) it dominates everything else the kernel does.
- A library that requires float at the boundary effectively locks streaming users out of high-rate modes on every SDR in the list. This is incompatible with serving the RF field generally.
- Other fixed-point-only contexts (space-qualified processors with no FPU, ITU-T G.191 codec test vectors, FPGA-coprocessor interop) reinforce the same conclusion.

Scope is bounded by importance, not by mechanism: every hot-path kernel that runs at sample rate gets a fixed-point implementation; everything else (filter design, FEC, channel models, spectral estimators that run once per buffer rather than per sample) stays float-only until a user proves otherwise.

At 0.0.1, the fixed-point sample types are:

- **`Complex<i16>`** — the dominant native kernel format across USRP, BladeRF, LimeSDR, Sidekiq, Pluto. The Q-format scaling (Q1.11 for BladeRF's `SC16_Q11`, Q1.15 for full-scale USRP `sc16`, intermediate fractional widths for 12- and 14-bit ADCs delivered in a 16-bit container) is *not* baked into the type; it's a property of the surrounding code's interpretation. Conversion helpers take the Q-format as a parameter (or are named with the Q-format) so the same `Complex<i16>` works for any radio.
- **`Complex<i8>`** — high-rate kernel format for BladeRF, USRP, and HackRF (where it's the only format).
- **`Complex<u8>` with DC midpoint at 127.5** — RTL-SDR. A **transport-only** type at the SDR boundary; not a sample type any kernel operates on. Rationale: `u8 + u8` is unsigned wrapping arithmetic on biased values, which is the wrong arithmetic for signal addition (two near-zero samples around 128 wrap to "maximum negative"). Rather than teach the `Sample` trait to do bias-aware arithmetic for one radio's format, the boundary helper debiases unconditionally — it takes a `&[Complex<u8>]` and produces a `&mut [Complex<i8>]` or `&mut [Complex<f32>]` with the bias subtracted. From that point on the rest of the library never sees a biased sample. Cost: one mandatory copy per RTL-SDR buffer (invisible at RTL-SDR's hobbyist rates and noise floor); benefit: every kernel's arithmetic semantics stay uniform and signed.

`Complex<i16>` and `Complex<i8>` implement `T: Sample` (via Rust's normal `i16` / `i8` types — signed arithmetic matches signal arithmetic modulo overflow). `Complex<u8>` does not implement `Sample` and is accepted only by boundary helpers, by design.

Boundary conversion helpers ship at 0.0.1: float-to-fixed and fixed-to-float for `Complex<i16>` and `Complex<i8>`, fixed-to-fixed/float for `Complex<u8>` (debiasing), with Q-format scaling as a parameter where applicable, and optional deinterleave-to-separate-arrays for users who want that shape. These are trivial code but they're what every SDR user needs immediately on receiving a sample buffer.

Fixed-point implementations at 0.0.1 of: FIR filtering, NCO/DDS, complex multiply, magnitude/power calculation. These are the hot-path kernels a streaming receiver spends most of its time in. Everything else remains float-only at 0.0.1.

The fixed-point footprint grows post-0.0.1 as users find kernels that need it. The `T: Sample` discipline (decision 5) guarantees this growth is non-breaking.

### Named Q-format constants per radio

A `Complex<i16>` from a USRP and a `Complex<i16>` from a BladeRF look identical at the byte level but mean different things numerically (Q1.15 vs Q1.11 — a 16× scaling factor). A user who owns several radios should not have to know this off the top of their head; making them pass a raw scaling number to a boundary helper is a footgun.

honeyeater therefore ships a `q_format` module of named constants, one per supported radio's wire format. The table below targets full coverage of every radio with a current SoapySDR support module, with the Q-format inferred from the driver's `fullScale` parameter (`fullScale == 2^n` ⇒ Q1.n in the signed container).

| Radio family | SoapySDR module | Native sample format | Constant | Q-format |
|---|---|---|---|---|
| Ettus / NI USRP (B/N/X/E/N3xx) | SoapyUHD | `CS16` / `CS8` / `CF32` (wire: `sc16` / `sc12` / `sc8`) | `USRP_SC16` / `USRP_SC8` | Q1.15 / Q1.7 (defaults; per-radio AD936x variants documented inline) |
| BladeRF (x40 / x115 / 2.0 micro), default 16-bit | SoapyBladeRF / libbladeRF `SC16_Q11` | `Complex<i16>`, fullScale 2048 | `BLADERF_SC16_Q11` | **Q1.11** (LSB-aligned in i16, 4 MSB headroom) |
| BladeRF 2.0 micro, packed 12-bit transport | libbladeRF `SC16_Q11_PACKED` | 12-bit packed over the wire; same Q1.11 to the user | `BLADERF_SC16_Q11_PACKED` | **Q1.11** (transport-equivalent to `SC16_Q11`) |
| BladeRF 2.0 micro, 122.88 MS/s oversample | libbladeRF `SC8_Q7` | `Complex<i8>`, fullScale 128 | `BLADERF_SC8_Q7` | **Q1.7** (the `libbladeRF.h` doc-comment for this format has a copy-paste bug — see Nuand issue #939 — the symbol name is authoritative) |
| HackRF One | SoapyHackRF | `Complex<i8>`, fullScale 128 | `HACKRF_SC8` | Q1.7 |
| RTL-SDR (RTL2832U) | SoapyRTLSDR | raw is `Complex<u8>`, the SoapySDR driver re-biases to `Complex<i8>` fullScale 128 | `RTL_SDR_U8` (raw biased) / `RTL_SDR_SC8` (post-bias) | unsigned 127.5 midpoint (raw) / Q1.7 (post-bias) |
| Airspy R2 / Mini | SoapyAirspy | `Complex<i16>`, fullScale 32767 | `AIRSPY_R2_SC16` | Q1.15 (12-bit ADC MSB-aligned in i16) |
| Airspy HF+ / Discovery | SoapyAirspyHF | `Complex<f32>` only | `AIRSPYHF_CF32` | n/a (float) |
| SDRplay RSP1A / RSPdx / RSPduo | SoapySDRPlay3 | `Complex<i16>`, fullScale 32767 | `SDRPLAY_SC16` | Q1.15 (14-bit ADC MSB-aligned) |
| ADALM-Pluto | SoapyPlutoSDR | `Complex<i16>`, RX fullScale 2048 / TX fullScale 32768 | `PLUTO_RX_SC16_Q11` / `PLUTO_TX_SC16_Q15` | **RX Q1.11 (LSB-aligned), TX Q1.15 (MSB-aligned)** |
| LimeSDR / LimeSDR-Mini / LimeSDR-USB | SoapyLMS7 | `Complex<i16>`, fullScale 32767 (also `CS12` packed) | `LIMESDR_SC16` (`LIMESDR_CS12` for packed) | Q1.15 (12-bit LMS7002M MSB-aligned in i16); packed 12-bit transports the same values |
| FUNcube Dongle Pro+ | SoapyFCDPP | `Complex<i16>` via ALSA (16-bit audio path) | `FCDPP_SC16` | Q1.15 |
| Epiq Solutions Sidekiq | SoapySidekiq | `Complex<i16>` | `SIDEKIQ_SC16` | not consistently documented per model — confirm against SDK manual at first integration |
| Mirics MSi2500 / MSi001 | SoapyMiri (community) | `Complex<i16>` | `MIRI_SC16` | not documented — confirm at integration |
| Red Pitaya STEMlab | SoapyRedPitaya | `Complex<i16>` over TCP | `REDPITAYA_SC16` | Q1.15 (14-bit ADC MSB-aligned) |
| Fairwaves XTRX (LMS7002M) | SoapyXTRX (community) | `Complex<i16>` | `XTRX_SC16` | Q1.15 (same family as LMS7) |
| Skylark Iris / Faros (massive-MIMO) | SoapyIris (community) | `Complex<i16>` | `IRIS_SC16` | Q1.15 |
| RFSpace NetSDR / Afedri | SoapyNetSDR / SoapyAfedri (community) | `Complex<i16>` | `NETSDR_SC16` | Q1.15 |
| gr-osmosdr umbrella (RFSpace, MiriSDR, etc.) | SoapyOsmo | varies | per underlying driver | per underlying driver |
| Sound-card / FunCube-class | SoapyAudio (community) | `Complex<f32>` from ALSA/PortAudio | n/a (float) | n/a |
| Network transport (not a radio) | SoapyRemote | passthrough | n/a | n/a |

User code reads as: `let floats = sdr::to_complex_f32(samples, q_format::BLADERF_SC16_Q11);` — they pick the constant for the radio they own and the right numerical scaling falls out. The table also lives as a rustdoc page so it's discoverable in the published docs.

This is **not** vendor-specific code (no driver bindings, no I/O paths, no licence entanglements) — just named constants and a docs table. It makes the bare `Complex<i16>` / `Complex<i8>` representation safe in practice: the Q-format-mixing bug only fires if a user types raw scaling numbers, and the constants make that unnecessary. The SoapySDR API itself does not standardise Q-format per radio — only `fullScale` — so this table is the canonical mapping; it should be updated whenever a new SoapySDR module appears or an existing one changes its native format.

**Open question for first implementation**: whether `Complex<i16>` and `Complex<i8>` are used directly, or whether they're wrapped in newtypes (`Sc16`, `Sc8` or similar) for type clarity at API boundaries. The trade-off: direct use is more familiar and composes more easily with other Rust crates; newtypes catch a residual class of "I passed USRP Q1.15 samples to a function expecting BladeRF Q1.11 samples" mistakes at compile time, even when the named constants above are used at conversion time. With the named constants in place the bare representation is likely sufficient, but the decision is deferred to first-kernel time; whichever choice is made should be consistent across both. (`Complex<u8>` is already decided: transport-only, no newtype needed because it never reaches a generic `Sample`-bounded API.)

### 7. Workspace versioning — synchronised across crates

All crates in the workspace (`honeyeater`, `honeyeater-core`, `honeyeater-test`, and future siblings) share one version number. Every release bumps every crate. This matches tokio's policy and makes the downstream audit story simple: a user pins `honeyeater 0.5.x` and gets a coherent set.

The cost is occasional dead version bumps in crates that didn't actually change; this is cheap relative to the audit-clarity benefit, especially for defence and aerospace users who care about exact-version pinning.

### 8. Error handling — mixed: panic on contract violations, `Result` on data-driven failures

Panic when the caller has violated the API contract — passed mismatched buffer lengths, asked for a filter of order zero, requested an FFT of length 1, etc. These are programmer bugs the caller should fix; turning them into `Result` values forces error-handling boilerplate on call sites that have no recovery path.

Return `Result<T, E>` (with a honeyeater error enum) when something can fail based on data the caller cannot validate up-front — filter design that doesn't converge to a stable design, file read that fails, SDR capture format that doesn't match what was expected. These are recoverable conditions and the caller deserves the chance to handle them.

This is the current consensus in the Rust scientific computing ecosystem (`ndarray`, `nalgebra`, `rustfft` all use it). It's also appropriate for honeyeater specifically because the Panop runtime runs as a long-lived process where unbounded panicking is unacceptable: data-driven errors must be recoverable.

A panic policy needs to be documented per public API. The first-kernel recipe (see roadmap) will set the precedent.

### 9. Concurrency — `Send` by default, `Sync` only where trivially correct, no internal parallelism at 0.0.1

honeyeater types implement `Send` (movable between threads) by default. This is the typical SDR pattern: a USB-reader thread receives buffers from the radio and hands ownership to a demodulation thread, which hands to a decoder thread. Each filter or NCO lives on exactly one thread at a time, but moves between threads at handoff points.

`Sync` (multiple threads using the same instance simultaneously) is opt-in per type and only implemented where it is trivially free — stateless function-objects, immutable lookup tables. Most stateful processors are not `Sync`, because making them so would require atomics or locking that the typical caller doesn't want to pay for.

No internal parallelism (no rayon, no thread pools, no parallel-by-default kernels) at 0.0.1. Parallelism is the runtime's job, not the library's. Adding `rayon`-based parallel versions of specific kernels later is non-breaking; building them in now and forcing users to opt out is.

### 10. Licence fence — ahead-of-time capture, library link graph stays permissive

Non-permissively-licensed reference implementations (libfec under LGPL, GNU Radio under GPL-3, MATLAB toolboxes as data-only) are used **only** in a separate `tools/oracle-gen/` workspace that exists outside the published crate set. That workspace runs the non-permissive references to generate reference vectors, which are then committed to `tests/vectors/` as opaque binary blobs with attribution.

The published library's link graph never touches non-permissive code, not even at test time. The library and its dev-dependencies are MIT/Apache/BSD throughout.

This is the same pattern `ring` uses for NIST CAVP cryptographic test vectors. It keeps honeyeater unambiguously redistributable under MIT-OR-Apache-2.0, with no LGPL-relinking obligation on downstream users.

## Style notes for future contributors and AI assistants

- Verify claims about other libraries before relying on them in a design decision. Reading source on GitHub or current docs beats memory or training data.
- Don't lean on audio libraries (JUCE, fundsp, etc.) as references. Their single-rate single-processor model doesn't match honeyeater's multi-RX/TX-chain case.
- liquid-dsp and scipy.signal are the primary references for "what does the field do here." liquid-dsp is permissively licensed so its decisions can be observed in source and its outputs used.
- The Panop downstream context (`docs/downstream-context.md`) shapes API decisions but doesn't enter the library — honeyeater stays a general-purpose library, not a Panop SDK.
