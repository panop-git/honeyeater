# honeyeater

[![crates.io](https://img.shields.io/crates/v/honeyeater.svg)](https://crates.io/crates/honeyeater)
[![docs.rs](https://img.shields.io/docsrs/honeyeater)](https://docs.rs/honeyeater)

A Rust library of digital signal processing primitives for radio-frequency and electrical signals. Filters, transforms, modulation, forward error correction, channel models. Targets CPU-side processing of sample streams from radios, digitisers, and simulators.

Version 0.0.1 prepared for the first kernel release. The API remains experimental. Stewarded by [Panop](https://www.panop.ai), the spectrum security company.

Phase 0 scaffolding is complete. Phase 1 steps 1–10 are implemented. The RustFFT
backend is public through `RustFftBackend`; PhastFT remains an internal
cross-validation backend. Fixed-point magnitude/power calculation required by
architecture decision 6 is also implemented.

Publication requires the complete stable/MSRV/nightly cross-platform CI matrix
to pass and the release packages to be verified.

For runnable examples, see the facade's [crate documentation](crates/honeyeater/src/lib.rs).

## Documentation

Documentation — what honeyeater is, the roadmap, and the testing methodology — is published at [honeyeater.dev](https://honeyeater.dev).

## Licence

Dual-licensed under either of

- the [MIT licence](LICENSE-MIT) (SPDX: `MIT`), or
- the [Apache Licence, Version 2.0](LICENSE-APACHE) (SPDX: `Apache-2.0`),

at your option. SPDX licence expression: `MIT OR Apache-2.0`.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
