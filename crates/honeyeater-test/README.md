# honeyeater-test

Dev-only test helpers for the honeyeater DSP library: the seven tolerance assertion macros, a `.npy` reference-vector loader, and a scipy subprocess helper for live cross-validation.

Published alongside the workspace at the shared version, this crate is intended as a dev-dependency for kernel validation, including downstream tests. It is not needed to process signals in production. The loaders and subprocess helper are implemented; kernel tests use committed fixtures. The subprocess helper requires Python, and oracle regeneration uses the pinned environment in `tools/oracle-gen/requirements.txt`. See `docs/policies.md` and `docs/roadmap.md` for the test methodology.

## Licence

Dual-licensed under [MIT](../../LICENSE-MIT) or [Apache-2.0](../../LICENSE-APACHE), at your option.
