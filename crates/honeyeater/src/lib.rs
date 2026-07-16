//! # honeyeater
//!
//! Digital signal processing primitives for radio-frequency and electrical
//! signals. Filters, transforms, modulation, forward error correction,
//! channel models. CPU-side processing of sample streams from radios,
//! digitisers, and simulators.
//!
//! This is the **facade crate** for honeyeater. It re-exports the public
//! surface of the library's component crates so downstream users can depend
//! on a single crate.
//!
//! ## Status
//!
//! Pre-v0.0.1. No DSP kernels are implemented yet. The currently re-exported
//! surface is the type substrate from [`honeyeater_core`] — the [`Sample`]
//! trait and the [`Complex`] re-export — on which Phase 1 kernels will be
//! built. See the repository `docs/roadmap.md` for the implementation plan.
//!
//! ## Licence
//!
//! Dual-licensed under MIT or Apache-2.0, at your option.

#![forbid(unsafe_code)]

pub use honeyeater_core::{Complex, Sample, num_complex};
