//! Core types for the honeyeater DSP library.
//!
//! This crate provides:
//!
//! - The [`Sample`] trait — the universal abstraction over the element type of a
//!   signal stream. Implemented for `f32`, `f64`, `i16`, and `i8`, and (via a
//!   blanket impl) for the [`num_complex::Complex<T>`] wrapping of each. See
//!   `docs/architecture-planning.md` decision 5 for the rationale behind the
//!   trait's shape and decisions 4 and 6 for the sample-type set.
//! - A re-export of [`num_complex`] so downstream code has a stable import
//!   path independent of `rustfft`'s version pinning.
//!
//! No DSP kernels live here yet; this crate is currently the type-and-trait
//! substrate that Phase 1 kernels will build on. See `docs/roadmap.md`.

#![forbid(unsafe_code)]

pub use num_complex;
pub use num_complex::Complex;

mod sample;
mod windows;

pub use sample::Sample;

// Brings hann and hamming functions into current scope from their new sub-modules
pub use windows::hamming::{hamming_window, hamming_window_periodic};
pub use windows::hann::{hann_window, hann_window_periodic};
