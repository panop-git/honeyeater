//! # honeyeater-cuda
//!
//! CUDA GPU backend for the honeyeater DSP library.
//!
//! This crate is a placeholder that reserves the name on crates.io. The GPU
//! backend is a deferred long-term goal (see the repository `docs/vision.md`,
//! "Long-term"); no implementation exists yet.
//!
//! Unlike the rest of the workspace, this crate does not forbid `unsafe`: a
//! CUDA FFI backend must write `unsafe extern` blocks to call the driver. See
//! its `Cargo.toml` for why the workspace lints are not inherited here, and
//! `docs/policies.md` ("`unsafe` code") for the workspace-wide policy.
//!
//! ## Licence
//!
//! Dual-licensed under MIT or Apache-2.0, at your option.
