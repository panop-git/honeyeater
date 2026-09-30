# honeyeater

Facade crate for the honeyeater DSP library. Re-exports the public surface of the library's component crates so users can depend on a single crate.

Version 0.0.1 is prepared for the first kernel release. The facade exposes windows, biquad and FIR filtering, NCO / DDS, float and fixed-point mixers, SDR conversions and Q-format constants, CRCs, and CCSDS RS(255,223) encoding. The FFT trait is public, while backend constructors remain test-only. The API is experimental; see the top-level README and `docs/roadmap.md` for release status.

## Licence

Dual-licensed under [MIT](../../LICENSE-MIT) or [Apache-2.0](../../LICENSE-APACHE), at your option.
