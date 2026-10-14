# Changelog

All notable changes to honeyeater are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) once 0.1.0 is published.

## [Unreleased]

## [0.0.1] - 2026-09-30

Source release preparation; publication is a separate step. The FFT backend
constructors remain test-only, and full release validation is still required.

### Added

- Hann, Hamming, Blackman-Harris, and Kaiser windows, including periodic forms.
- RBJ low-pass biquad design and floating-point execution.
- FFT backend trait with PhastFT and RustFFT implementations validated in
  core tests against SciPy at 120 dB SNR (f64) and 60 dB (f32).
- CRC-32C (Castagnoli) and CRC-16/ARC with reveng catalogue check values.
- Float and fixed-point NCO / DDS, FIR execution, complex multiply and mixers.
- Q-format-aware SDR sample conversions, RTL-SDR midpoint handling,
  deinterleave/interleave helpers, and named radio-format constants.
- CCSDS RS(255,223) encoder, checked against pinned libfec codeword fixtures
  and CCSDS Annex F basis-transformation examples.
- Implemented `.npy` loaders and development-time Python subprocess helper.
- Committed reference fixtures and Python generators, with NumPy 2.5.1 and
  SciPy 1.18.0 pinned centrally and libfec pinned by source commit.

### Changed

- The facade re-exports the complete public core API.
- Workspace crates and internal dependency requirements move together to 0.0.1.
- Project and crate documentation describe implemented coverage, experimental
  API status, and remaining release gates.
- Oracle-generation instructions use one canonical Python requirements file;
  the redundant windows requirements file is removed.

### Fixed

- Window, biquad, and FFT fixture paths resolve from the generator source,
  allowing regeneration from the repository root.
- Window regeneration reports the correct number of output vectors.
- Ten FFT/window fixtures refreshed to reproduce with the canonical oracle pins
  (maximum absolute change below 2.3e-15).
- Phase 1 formatting, FFT test variable naming, and Python-helper error docs.

## [0.0.0] - 2026-07-17

### Added

- Version discipline in CI: the root `Cargo.toml` `[workspace.package]`
  version is the single source of truth; CI checks that `CHANGELOG.md`'s
  newest release heading matches it, and that every functional PR into
  `main` increments it (docs-only diffs are detected automatically;
  other non-functional changes use the `non-functional` PR label).
- Initial repository skeleton.
- Phase 0 scaffolding (per `docs/roadmap.md`):
  - Cargo workspace with four member crates: `honeyeater` (facade),
    `honeyeater-core` (sample types, traits), `honeyeater-test`
    (tolerance assertion macros and oracle helpers), and `honeyeater-cuda`
    (a placeholder reserving the name for the deferred GPU backend).
  - `Sample` trait (`honeyeater-core`) with impls for `f32`, `f64`, `i16`,
    `i8`, and `Complex<…>` of each.
  - `num-complex` re-export through `honeyeater-core` for stable downstream
    import paths.
  - `rustfft` dependency wired in `honeyeater-core` (used by the Phase 1
    FFT wrapper).
  - Seven tolerance assertion macros in `honeyeater-test` (`assert_close!`,
    `assert_snr_db!`, `assert_bit_exact!`, `assert_spectral_mask!`,
    `assert_ber_at_ebn0!`, `assert_parseval!`, `assert_distribution_ks!`).
  - `.npy` loader and scipy-subprocess helper signatures (initial stubs;
    implementations deferred to the first kernel that needs them).
  - Separate `tools/oracle-gen/` workspace for non-permissive oracle
    runners (libfec etc.), kept out of the library's link graph.
- Repository hygiene:
  - `docs/policies.md` documenting cross-cutting policies (`unsafe` forbid,
    clippy pedantic, licence allowlist, MSRV, panic vs `Result`,
    deprecation).
  - `CONTRIBUTING.md`, `CODE_OF_CONDUCT.md`, `SECURITY.md`.
  - `.github/workflows/ci.yml` running `cargo fmt`, `cargo clippy -D
    warnings`, `cargo test`, `cargo doc` (with `RUSTDOCFLAGS=-Dwarnings`),
    `cargo deny check`, and `mdbook build` across stable, MSRV (1.91), and
    nightly on Linux, macOS, and Windows.
  - `deny.toml`, `clippy.toml`, `rustfmt.toml` with rationale in
    `docs/policies.md`.
- Workspace MSRV pinned to **Rust 1.91** (latest stable minus two; see
  `docs/architecture-planning.md` decision 1).
- Contribution-licensing and DCO policy: contributions are accepted under
  the project's `MIT OR Apache-2.0` dual licence (inbound = outbound; see
  the README "Licence" section), every commit must be signed off against
  the Developer Certificate of Origin 1.1 (`DCO.txt`), and CI fails any
  commit lacking a `Signed-off-by` trailer (`docs/policies.md`
  "Contribution licensing and sign-off (DCO)"; `CONTRIBUTING.md`
  "Licensing and sign-off").

### Changed

- `CODE_OF_CONDUCT.md` replaced: the adopted Rust Code of Conduct promised
  a staffed moderation and report-handling process the project does not
  offer. Now a short house-written statement — professional conduct
  expected, discretionary curation with GitHub's standard tools, concerns
  raised via the issue tracker. Rationale recorded at `docs/roadmap.md`
  Phase 0 step 7.
