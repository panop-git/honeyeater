# Contributing to honeyeater

honeyeater is pre-v0.0.1. Expect APIs, crate layout, and this document to change without notice until the milestone in `docs/roadmap.md` (CCSDS Reed-Solomon (255, 223) encoder, bit-exact) cuts 0.0.1.

## Read these first

- [`docs/vision.md`](docs/vision.md) — what honeyeater is and is not. Scope, principles, comparators.
- [`docs/roadmap.md`](docs/roadmap.md) — the implementation plan, kernel ordering, tolerance vocabulary, and per-category oracle stack. The source of truth for "how do we test this?" and "is this on the milestone path?"
- [`docs/architecture-planning.md`](docs/architecture-planning.md) — numbered design decisions with rationale. If you want to deviate from one, the deviation must update that file with reasoning.
- [`docs/policies.md`](docs/policies.md) — cross-cutting policies: `unsafe`, clippy posture, licence allowlist, MSRV, panic vs `Result`, deprecation. These are not negotiable per PR; if you want to change one, propose the policy change first.

## Previewing the documentation locally

The documents above are published as a book at [honeyeater.dev](https://honeyeater.dev), built from the `docs/` directory with [mdBook](https://rust-lang.github.io/mdBook/). To preview documentation edits with live reload:

```sh
cargo install mdbook    # one-time; compiles from source, takes a few minutes
mdbook serve --open     # run from the repository root
```

The book opens at `http://localhost:3000` and rebuilds automatically whenever a file in `docs/` changes.

If `cargo` is not on your machine yet, install the Rust toolchain with [rustup](https://rustup.rs) — you will need it to build the library anyway. `cargo install` fetches a tool from crates.io, compiles it, and puts the binary on your `PATH` (in `~/.cargo/bin`). Any recent mdBook release is fine for previewing; CI separately verifies the book builds against a pinned version.

## How a change lands

1. **Confirm scope.** Tier-1 primitives (`docs/roadmap.md`, Phase 1) land in 0.0.1; Tier-2 specialty primitives are post-0.0.1; some categories are excluded on principle (`docs/vision.md`, "Exclusion on principle"). If your change crosses a boundary, raise it before implementing.
2. **Write the test first.** Every kernel is validated against a named oracle from `docs/roadmap.md` ("Oracle stack by module category"). The tolerance macros are in `honeyeater-test` (`docs/policies.md`, "Clippy posture" also covers why pedantic catches the silent-cast bugs DSP code is prone to). If you find a kernel that needs a measure the seven macros don't cover, that's a `docs/roadmap.md` update first.
3. **Implement the kernel.** Generic over `T: Sample` where it makes sense; both float and fixed-point where the roadmap calls for it (`docs/architecture-planning.md` decision 6).
4. **Run the local checks.** Before pushing:

   ```sh
   cargo fmt --all
   cargo clippy --workspace --all-targets -- -D warnings
   cargo test --workspace --all-targets
   cargo doc --workspace --no-deps
   cargo deny check
   ```

   CI runs the same set across stable, the pinned MSRV, and nightly. Local stable is usually enough; the MSRV check catches "I used a feature added in a newer Rust" and the nightly check catches future lint changes early.
5. **Bump the version and update CHANGELOG.md.** The `[workspace.package]` version in the root `Cargo.toml` is the single source of truth — every crate inherits it, and CI enforces the rules here. A functional PR increments it (patch bump pre-0.1.0) and records its changes under a matching `## [<new version>]` heading in `CHANGELOG.md`, using the Keep-a-Changelog sections (`Added`, `Changed`, `Deprecated`, `Removed`, `Fixed`, `Security`). Two exemptions from the bump: docs-only diffs (CI detects these automatically), and other non-functional changes, which a maintainer marks with the `non-functional` PR label. Notes that don't belong to any version yet can sit under `## [Unreleased]` until a functional PR rolls them into its version section.
6. **Open the PR.** One change per PR. Smaller PRs land faster.

## What this project does not accept

- **`unsafe` code in any honeyeater crate.** See `docs/policies.md`, "`unsafe` code". `unsafe` is mechanically forbidden at the workspace level. If you genuinely need it (e.g. a future hand-tuned SIMD crate), open a policy-change discussion first; don't try to land the code and the policy change in the same PR.
- **New dependencies with non-permissive licences.** The licence allowlist is enforced by `cargo-deny`. If your change pulls in a copyleft transitive dep, the build fails and the dep needs to be replaced or removed.
- **Audio-perceptual kernels.** Loudness, dynamics, reverb, codec primitives, pitch detection — out of scope on principle. There is an existing Rust library that serves audio-perceptual DSP; honeyeater serves RF and electrical.
- **Waveforms defined only in restricted military standards** or ITAR/restricted-EAR controlled items. See `docs/vision.md`, "Exclusion on principle".
- **Internal threading or async** at this stage. Parallelism is the runtime's job (`docs/architecture-planning.md` decision 9).

## Test discipline

A kernel without a test against a named oracle is not a kernel honeyeater accepts. The named-oracle requirement is structural to the library's promise; ad-hoc "this looks right to me" tests don't make the bar.

When the oracle is permissively licenced (scipy, liquid-dsp, AFF3CT), the cross-validation typically runs live via `honeyeater_test::scipy::run` or a vendored binding. When the oracle is non-permissively licenced (libfec, GNU Radio), it runs out-of-band in `tools/oracle-gen/` (`docs/architecture-planning.md` decision 10) and the captured vectors are committed as opaque blobs to `tests/vectors/`.

The seven tolerance macros are listed and documented in `crates/honeyeater-test/src/macros.rs`. Pick the one that fits your kernel's class. Don't invent new measures without first proposing the addition to `docs/roadmap.md`'s tolerance vocabulary.

## Style

- Code is formatted with `rustfmt`; see `rustfmt.toml`.
- Clippy posture is `pedantic` warn — comply, don't silence. Exceptions go in `clippy.toml` (thresholds) or `[workspace.lints.clippy]` (lint level) with a written rationale in `docs/policies.md`.
- Doc-comments: every public item has rustdoc. Functions that can panic document the conditions under `# Panics`. Functions returning `Result` document the error conditions under `# Errors`.
- Commit messages: imperative mood, short summary line, `Signed-off-by` trailer (`git commit -s`; see "Licensing and sign-off"). Reference the roadmap step or architecture decision being addressed when relevant.
- Never commit absolute local filesystem paths (`/home/...`, `/Users/...`, `C:\Users\...`) in code, docs, or config — they leak the author's environment and persist in history. Refer to locations relative to the repository root.

## Licensing and sign-off

1. **Inbound = outbound.** Contributions are submitted under the project's dual licence, MIT OR Apache-2.0 — the same terms honeyeater ships under. Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 licence, shall be dual licensed as above, without any additional terms or conditions.
2. **Sign off every commit** with `git commit -s`. This appends a `Signed-off-by: Your Name <you@example.com>` line certifying the [Developer Certificate of Origin 1.1](DCO.txt): that you wrote the change, or otherwise have the right to submit it under the project licence.
3. **Unsigned commits will not be merged.** CI fails any commit lacking a valid `Signed-off-by` trailer. To fix a forgotten sign-off: `git commit --amend -s` for the last commit, `git rebase --signoff <base>` for a range, then force-push your branch.

## Reporting security issues

See [`SECURITY.md`](SECURITY.md). Do not open a public issue for a suspected vulnerability.

## Code of conduct

See [`CODE_OF_CONDUCT.md`](CODE_OF_CONDUCT.md). The summary: be civil, be precise, attack arguments not people.
