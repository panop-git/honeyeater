# honeyeater-core

Core types and Phase 1 primitives for the honeyeater DSP library: windows, an RBJ low-pass biquad, FIR execution, NCO / DDS, float and fixed-point complex multiply and mixers, SDR sample conversions, radio Q-format constants, CRCs, and CCSDS RS(255,223) encoding, alongside the `Sample` trait and `num-complex` re-export.

Version 0.0.1 is prepared for the first kernel release. The API remains experimental. FFT backend implementations are validated in tests, but only the backend trait is currently public. See the top-level README and `docs/roadmap.md` for release gates.

## Licence

Dual-licensed under [MIT](../../LICENSE-MIT) or [Apache-2.0](../../LICENSE-APACHE), at your option.
