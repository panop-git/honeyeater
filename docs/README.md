# honeyeater documentation

This is the narrative documentation for the **honeyeater** DSP library: a Rust library of digital signal processing primitives for radio-frequency and electrical signals.

New to the project? Begin with **[Start here](start-here.md)** — a gentle, no-background-assumed introduction. The rest of the documentation is split into three sections.

**Getting started:**

- **[Start here](start-here.md)** — what honeyeater is for and the handful of terms you need, explained from scratch.

**Project** — what honeyeater is and how it is built:

- **[Vision](vision.md)** — what honeyeater is, what it isn't, and the design principles that follow. The fuller, slightly denser statement once Start here has oriented you.
- **[Roadmap](roadmap.md)** — the test-driven implementation plan, phased kernel ordering, tolerance vocabulary, and oracle stack per kernel category. The source of truth for "is this the right next step?" and "what oracle do I validate this kernel against?"
- **[Architecture decisions](architecture-planning.md)** — numbered design decisions with rationale. The reference document for "why is the API like this?"
- **[Policies](policies.md)** — cross-cutting policies (`unsafe` stance, clippy posture, licence allowlist, contribution licensing and DCO sign-off, MSRV, panic vs `Result`). These apply across the whole codebase and across all contributors.
- **[Downstream context](downstream-context.md)** — what specific downstream use shapes the library's API. The most-cited example is the Panop flowgraph runtime; honeyeater itself remains general-purpose.

**Using honeyeater** — practical guides for working with the library:

- **[Testing](testing.md)** — the seven tolerance assertion macros from `honeyeater-test`, how to use each, default thresholds per kernel class, and patterns for testing filters, FEC, and stochastic kernels.

**Reference:**

- **[Glossary](glossary.md)** — plain-English definitions of the recurring DSP, RF, and Rust terms used across these pages. Look here whenever a term is unfamiliar.

## Status

honeyeater is pre-v0.0.1. No DSP [kernels](glossary.md#kernel) (individual signal-processing building blocks) are implemented yet.

## API reference

The auto-generated API reference ([rustdoc](glossary.md#rustdoc), Rust's built-in documentation generator) is the source of truth for what types and functions actually exist. Build it with `cargo doc --workspace --open`.

## Licence

Dual-licensed under [MIT](https://github.com/panop-git/honeyeater/blob/main/LICENSE-MIT) or [Apache-2.0](https://github.com/panop-git/honeyeater/blob/main/LICENSE-APACHE), at your option. Contributions are accepted under the same terms (inbound = outbound), and every commit must be signed off certifying the Developer Certificate of Origin (`DCO.txt` at the repository root) — see [Policies](policies.md#contribution-licensing-and-sign-off-dco).
