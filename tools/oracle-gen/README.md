# oracle-gen

`tools/oracle-gen/` is a **separate Cargo workspace** from the main honeyeater library. Its purpose is to host reference-vector generators that depend on non-permissively-licensed oracles — primarily KA9Q libfec (LGPL) for FEC vectors, and possibly AFF3CT (MIT, but heavy) for LDPC and BCH curves.

## Why this is its own workspace

Architecture decision 10 (`docs/architecture-planning.md`) commits the honeyeater library to a clean permissive licence story: `MIT OR Apache-2.0`, no copyleft anywhere in the library's link graph, including its dev-dependencies.

LGPL-licensed code (libfec is the typical example) cannot enter that link graph without imposing relinking obligations on downstream users. But libfec is the de-facto numerical reference for CCSDS Reed-Solomon, and the test methodology demands we validate against it.

The resolution is to run libfec (and any other non-permissive oracle) here, in a separate workspace that produces **binary reference vectors** as output. Those vectors are committed to `crates/honeyeater-test/tests/vectors/` (or under the consuming kernel crate's `tests/vectors/`) as opaque blobs with attribution. The library's link graph never touches LGPL code; only the developer running an oracle-gen binary does.

This mirrors the pattern that the `ring` crate uses for NIST CAVP cryptographic test vectors.

## Status

Phase 0 placeholder. No real oracle generators exist yet. The first generator will be a libfec wrapper producing CCSDS Reed-Solomon (255, 223) encoder vectors for Phase 1 step 10 (see `docs/roadmap.md`).

## How to use this workspace

`oracle-gen/` is not part of the main workspace. You build it explicitly:

```sh
cargo build --manifest-path tools/oracle-gen/Cargo.toml
```

CI does not run any oracle-gen binaries by default. The expectation is that a developer regenerates vectors when an oracle version pin changes, commits the regenerated vectors, and CI validates honeyeater's kernels against the committed blobs.

Per-generator pins (libfec git commit, AFF3CT version, scipy version) will be documented in each generator's own README under this directory.
