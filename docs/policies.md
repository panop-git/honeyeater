# honeyeater — policies

Cross-cutting policies that apply to the whole codebase. Where a policy has an underlying architectural decision, this document summarises the policy and links to the decision in `docs/architecture-planning.md`.

This file is the source of truth for contributor-facing rules. CONTRIBUTING.md is the front door to the project; it points here for the substance.

## `unsafe` code

honeyeater's DSP crates contain **no `unsafe` code**, mechanically enforced by `unsafe_code = "forbid"` in the workspace lints, which every DSP crate inherits.

The sole exception is `honeyeater-cuda` (a placeholder for the deferred GPU backend, `docs/vision.md` "Long-term"): a CUDA FFI backend must write `unsafe extern` blocks to call the driver, so that crate opts out of the workspace lints and permits `unsafe`. The exception is confined to the GPU crate, which no other crate depends on; the core DSP link graph stays `unsafe`-free.

`forbid` is the strongest possible level: the compiler will refuse to build any crate in this workspace that contains an `unsafe` block. The attribute cannot be overridden by an inner `#[allow]`; bypassing it requires editing the crate root itself, which makes any future relaxation of the policy a deliberate, public, code-review-visible change.

**Why.** Vision principle #3 (`docs/vision.md`) is "memory-safe by construction." The class of bugs that dominate C DSP libraries — buffer overruns, use-after-free, data races on mutable state — is the reason a meaningful share of honeyeater's target users picked Rust. Allowing internal `unsafe` would compromise that claim. A policy of `forbid` keeps the claim mechanically true and avoids the slow drift toward "well, just this one block" that softer policies invite.

**What this does not affect.** `unsafe` in *dependencies* is fine. `rustfft` uses `unsafe` extensively for its SIMD kernels; that is rustfft's policy, not ours. The `forbid(unsafe_code)` attribute is per-crate, not transitive. Standard library, `num-complex`, and any other dependency operates under its own policy.

**Future SIMD work.** If honeyeater ever wants hand-tuned SIMD intrinsics of its own (not delegated to a SIMD-aware dependency), the policy will need to be revisited. The relaxation should not be a quiet `#[allow]`; it should be either (a) a separate crate that opts in with its own attribute and `clippy::undocumented_unsafe_blocks = "deny"` enforcing `// SAFETY:` comments on every block, or (b) a documented amendment to this policy. Either way, the conversation is loud. This is the point of using `forbid` rather than `deny`.

## Clippy posture

Lints are configured workspace-wide via `[workspace.lints]` in the root `Cargo.toml`, with each member crate doing `[lints] workspace = true`.

The posture:

- `clippy::pedantic` is set to `warn` (so all pedantic lints fire as warnings, and CI treats warnings as errors).
- `clippy::nursery` stays `allow` (off). Nursery contains experimental lints with known false-positive rates that aren't worth fighting.
- `clippy::cargo` is set to `warn` (catches metadata issues in `Cargo.toml`).
- A handful of pedantic lints with high noise and low value are explicitly silenced — currently `clippy::module_name_repetitions`.

**Why pedantic.** DSP code's single most error-prone pattern is silent numerical casting: `i32 as f32` quietly loses precision, `f32 as i16` quietly truncates, `==` on floats is almost always wrong. Default clippy is silent on most of these; pedantic catches them all (`cast_precision_loss`, `cast_possible_truncation`, `cast_sign_loss`, `float_cmp`, `unreadable_literal`, etc.). For a library whose entire job is numerical work — and whose fixed-point story (architecture decision 6) is full of integer↔float conversions where Q-format scaling matters — these are exactly the warnings that should fire.

**Why comply rather than silence.** Pedantic produces friction: it will suggest `#[must_use]` on every getter, demand `# Errors` and `# Panics` docs on `Result`-returning and panicking functions, and occasionally object to legitimate patterns. The friction is mostly correct. honeyeater is modular by design — most public functions are pure — so the bug-class `must_use_candidate` protects against (calling a pure function and discarding its result) is unambiguously a bug when it occurs, and the cost of complying with the lint is the cost of typing `#[must_use]`. The library's stance is to take that cost.

**The one silenced lint.** `clippy::module_name_repetitions` complains when a type inside `mod fir` is named `FirCoefficients` (it wants `Coefficients`). Sometimes that is right; often the qualified name is more discoverable in [rustdoc](glossary.md#rustdoc) and at use sites. The rule is silenced workspace-wide; individual cases that violate the spirit are caught in review.

Other pedantic lints may be silenced over time when concrete evidence accumulates that they cost more than they catch. Each silencing must come with a written rationale in this section.

## Dependency policy and licences (cargo-deny)

The workspace is policed by `cargo-deny`, configured in `deny.toml`. CI fails the build on violations.

### Licence allowlist

honeyeater is dual-licensed `MIT OR Apache-2.0`. Both are permissive: downstream consumers, including closed-source commercial users, can use honeyeater without copyleft obligations. That story only holds if every crate in the dependency tree is also permissive.

Allowed licences:

| Licence | Notes |
|---|---|
| `MIT` | Most common permissive licence. |
| `Apache-2.0` | Permissive with explicit patent grant. honeyeater's own. |
| `Apache-2.0 WITH LLVM-exception` | Apache-2.0 with the LLVM linking-exception clause; used by several core Rust crates. |
| `BSD-2-Clause`, `BSD-3-Clause` | Older permissive licences. |
| `ISC` | Simplified BSD-style; common in crypto and network crates. |
| `Unicode-DFS-2016`, `Unicode-3.0` | Required for `unicode-ident` (used by the Rust compiler itself) and its modern replacement. |
| `Zlib` | Permissive; used by zlib ports. |
| `0BSD` | Public-domain-equivalent; used by small utility crates. |
| `CC0-1.0` | Public-domain dedication; used by some data crates. |

Notably **excluded**:

- All GPL family (GPL-2.0, GPL-3.0, LGPL-2.1, LGPL-3.0, AGPL-3.0) — copyleft obligations.
- **MPL-2.0** — file-level copyleft. Much weaker than LGPL, but still creates a publication obligation on modifications that doesn't fit a "MIT OR Apache-2.0, full stop" story. Excluded by default; if a future dep genuinely needs it, the failing build forces the decision into the open.
- CDDL, SSPL, Elastic-2.0, BUSL-1.1 — various flavours of copyleft, source-available, or anti-cloud licences.
- "Custom" / unrecognised licences — must be reviewed individually.

LGPL specifically affects the FEC-oracle story (architecture decision 10): libfec is LGPL and is used **only** in `tools/oracle-gen/`, a separate workspace whose outputs (binary reference vectors) are committed to `tests/vectors/`. The published library's link graph never touches LGPL code. The `cargo-deny` licence check enforces this mechanically.

### Other `cargo-deny` checks

- **Advisories.** RustSec advisory-database check enabled. Yanked crates fail the build (`yanked = "deny"`).
- **Bans.** Multiple versions of the same crate warn but do not fail (`multiple-versions = "warn"`). Wildcard version requirements (`crate = "*"`) deny.
- **Sources.** Crates must come from crates.io. Unknown registries and arbitrary git sources are denied.

## Contribution licensing and sign-off (DCO)

Contributions are accepted **inbound = outbound**: unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in honeyeater, as defined in the Apache-2.0 licence, is dual licensed `MIT OR Apache-2.0` — the same terms the library ships under — without any additional terms or conditions. The canonical paragraph is in the README's "Licence" section.

Every commit must carry a `Signed-off-by:` trailer (`git commit -s`), certifying the [Developer Certificate of Origin 1.1](https://developercertificate.org/) — reproduced verbatim at the repository root as `DCO.txt` — that the contributor wrote the change or otherwise has the right to submit it under the project licence. CI fails any commit lacking a valid trailer; unsigned commits are not merged. The mechanics, including the fix for a forgotten sign-off, are in CONTRIBUTING.md ("Licensing and sign-off").

**Why.** The licence allowlist above keeps the *outbound* story clean: everything honeyeater ships is permissively licensed all the way down. This policy keeps the *inbound* side equally clean: every contributor's copyright enters the project under the terms it ships under, with per-commit provenance, and without CLA overhead. It is the model used by the Linux kernel (where the DCO originates) and GitLab, so it is familiar to contributors and legible to downstream compliance review.

## MSRV (minimum supported Rust version)

The MSRV is **the Rust stable release approximately three months prior to the most recent stable** — i.e. the latest-minus-two minor versions across Rust's six-week cadence. This is captured as `rust-version` in `[workspace.package]`.

This balances toolchain lag in regulated and corporate environments (where the freshest stable is often unavailable for months) against not freezing on genuinely old compilers (which would limit honeyeater's access to language and library improvements).

See architecture decision 1 for the rationale and comparison to other ecosystem patterns (tokio uses ~N-4; serde leaves it open; the ecosystem has no consensus).

**Practical implication.** Code in honeyeater must compile on the pinned MSRV. CI runs a build job on the pinned MSRV alongside stable and nightly. When a new MSRV is adopted, it counts as a minor breaking change and gets a CHANGELOG entry.

## Panic vs `Result`

honeyeater mixes the two deliberately. The policy is documented in full at architecture decision 8; the short form for contributors is:

- **Panic** when the caller has violated the API contract — mismatched buffer lengths, filter order zero, FFT of length 1, etc. These are programmer bugs the caller should fix.
- **Return `Result<T, E>`** when the failure is data-driven and the caller cannot validate up-front — filter design that doesn't converge, file read that fails, SDR capture format mismatch. These deserve to be recoverable.

Every public API documents its panic conditions in a `# Panics` rustdoc section, and its error conditions in `# Errors`. The pedantic lints `missing_panics_doc` and `missing_errors_doc` enforce this.

## Workspace versioning

All crates in the workspace share one version number. Every release bumps every crate, including crates with no changes in that release. See architecture decision 7.

This affects how releases are made (one version bump, not per-crate) and how downstream users pin (`honeyeater = "0.5"` gets a coherent set across all member crates).

## Deprecation and breaking changes

Pre-0.0.1, the codebase is unstable and breaking changes are free. CHANGELOG.md tracks them but nothing more is required.

Once 0.1.0 ships:

- Breaking changes require a minor version bump (per semver pre-1.0 convention).
- Deprecated items get `#[deprecated(since = "x.y.z", note = "...")]` and remain for at least one minor cycle before removal.
- Removal of a deprecated item is itself a breaking change requiring a minor bump.

This is conventional Rust ecosystem practice. The policy will be expanded as 0.1.0 approaches and real public surface accumulates.
