# honeyeater

[![crates.io](https://img.shields.io/crates/v/honeyeater.svg)](https://crates.io/crates/honeyeater)
[![docs.rs](https://img.shields.io/docsrs/honeyeater)](https://docs.rs/honeyeater)

A Rust library of digital signal processing primitives for radio-frequency and electrical signals. Filters, transforms, modulation, forward error correction, channel models. Targets CPU-side processing of sample streams from radios, digitisers, and simulators.

Version 0.0.1 prepared for the first kernel release. The API remains experimental. Stewarded by [Panop](https://www.panop.ai), the spectrum security company.

The working tree implements Hann, Hamming, Blackman-Harris and Kaiser windows, an RBJ low-pass biquad, CRC-32C and CRC-16/ARC, NCO / DDS, SDR sample conversions and radio Q-format constants, FIR execution, complex multiply and mixers, and a CCSDS Reed-Solomon (255,223) encoder. NCO, FIR, and mixer primitives support float and fixed-point samples. These APIs are available through the `honeyeater` facade.

FFT backends are validated against SciPy, but their constructors remain test-only; the facade currently exposes the `FftWrapper` contract. See the [roadmap](docs/roadmap.md) for the release gates. This change prepares source version 0.0.1; it does not publish it to crates.io.

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
