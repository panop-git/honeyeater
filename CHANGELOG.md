# Changelog

All notable changes to honeyeater are documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/), and the project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html) once 0.1.0 is published.

## [Unreleased]

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
  - `.npy` loader and scipy-subprocess helper signatures (stubs for now;
    implementations deferred to first kernel that needs them).
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
