# Downstream context

honeyeater is open-source and general-purpose, but its design is informed by specific downstream use at Panop. This document captures the constraints those use cases impose. The library itself contains no Panop-specific code or APIs, and no SDR-vendor-specific code or APIs — see `docs/architecture-planning.md` for the *general* sample-format landscape that drives the fixed-point story.

## The Panop flowgraph runtime

honeyeater will be the DSP layer underneath a GNU Radio-style flowgraph runtime used internally at Panop for RF processing. The runtime is a separate project, not part of honeyeater. honeyeater stays a library of primitives.

Two properties of the runtime constrain honeyeater's design:

1. **Compile-time graph construction.** Different waveforms (AIS receiver, DVB-S2 receiver, ADS-B receiver, etc.) are pre-compiled as separate graph artefacts. Waveform hot-swap is "stop running graph A, start running graph B" — both already-compiled and ready to run — not "reconfigure graph A's blocks in place." Switching is fast because there's nothing to compute at the moment of switch.

2. **Multiple coexisting RX/TX chains.** The runtime can run several independent graphs at different sample rates simultaneously: one wideband spectrum survey at high rate alongside a narrowband decoder at low rate. honeyeater must not embed any assumption that there's one global sample rate, one global configuration, or one graph in flight.

### What this rules in and out for honeyeater

- **No `prepare(spec)` step separated from construction.** A JUCE-style two-stage configuration is the wrong shape: the spec is known at compile time and the compiler can constant-fold every parameter. Processors take their parameters at construction. (See `docs/architecture-planning.md` decisions 3 and 3a.)
- **No stream-tag mechanism inside honeyeater.** GNU Radio attaches metadata (sample rate, time, frequency) to streams as offset-indexed tags. honeyeater doesn't do this — that's runtime infrastructure. The runtime owns connection topology, tag propagation, and rate-change handling. honeyeater just provides processors with known parameters baked in.
- **Per-block parameter mutability where it's safe.** A few parameters are routinely retuned in operational radios: NCO frequency, AGC reference level, gain. Those blocks expose mutator methods so the runtime can adjust them without rebuilding the graph. Parameters whose change would cause numerical transients (FIR coefficients) or require re-derivation (filter prototypes) are construction-only.
- **No threading, scheduling, or runtime infrastructure of any kind in honeyeater itself.** No internal thread pools, no rayon, no async. Concurrency is the runtime's job.

## Panop's hardware: high-rate fixed-point matters

Panop uses Nuand BladeRF SDRs in production, including the radio's 122.88 MS/s oversample mode where samples arrive as 8-bit fixed-point integers. At that rate (~8 ns per sample on one core), boundary conversion to f32 eats too much budget; the hot-path kernels must process native int8.

This is one concrete example of why honeyeater commits to native fixed-point processing at 0.0.1. It is **not** the only reason — the general SDR sample-format landscape (`docs/architecture-planning.md`, "Hardware: SDR sample-format landscape") shows that every major SDR family delivers integer samples natively, and the same boundary-conversion cost argument applies to USRP, LimeSDR, Sidekiq, HackRF, and others operating at high rates.

honeyeater handles this with general-purpose fixed-point types (`Complex<i16>`, `Complex<i8>`, `Complex<u8>`) plus boundary conversion helpers, all of which work uniformly across SDR vendors. There is no BladeRF-specific code path; there's no USRP-specific code path either. honeyeater treats every SDR's samples identically once they arrive as one of those three integer types.

## What this document is and isn't

This is design context, not API surface. Nothing here defines what honeyeater exports. honeyeater's published surface is general-purpose; the Panop runtime is one consumer it's designed to support well, but the library has no Panop-specific exports, no BladeRF-specific exports, no USRP-specific exports, and no flowgraph types.

SDR-specific bindings (a `bladerf-rs` crate, a `uhd-rs` crate, the integration with SoapySDR, the Panop flowgraph) all live in separate crates that depend on honeyeater. They are not part of honeyeater.
