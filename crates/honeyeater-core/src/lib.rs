//! Core types and Phase 1 DSP primitives for the honeyeater library.
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
//! Version 0.0.1 includes the window family, RBJ low-pass [`Biquad`],
//! [`FirFilter`], [`Nco`], float and fixed-point mixer and magnitude/power
//! primitives, [`q_format`] and SDR boundary helpers, CRCs, the public
//! [`RustFftBackend`], and the CCSDS RS(255,223) encoder.
//! The API remains experimental.

#![forbid(unsafe_code)]

pub use num_complex;
pub use num_complex::Complex;

mod biquad;
mod crc;
mod fft;
mod fir;
mod magnitude;
mod mixer;
mod nco;
pub mod q_format;
mod rs_ccsds;
mod sample;
mod sdr_boundary;
mod windows;

// Brings magnitude functions into current scope
pub use magnitude::{
    MagnitudeSample, complex_magnitude, complex_magnitudes,
    complex_magnitudes_owned, complex_power, complex_powers,
    complex_powers_owned,
};

pub use sample::Sample;

// Brings window functions into current scope from their sub-modules
pub use windows::blackmanharris::{blackmanharris_window, blackmanharris_window_periodic};
pub use windows::hamming::{hamming_window, hamming_window_periodic};
pub use windows::hann::{hann_window, hann_window_periodic};
pub use windows::kaiser::{kaiser_window, kaiser_window_periodic};

// Brings CRC functions into current scope
pub use crc::{crc16_arc, crc32_castagnoli};

// Brings Biquad filter functions into current scope
pub use biquad::Biquad;

// Brings FFT function into current scope
pub use fft::{FftWrapper, RustFftBackend};

// Brings complex multiply and mixer primitives into current scope
pub use mixer::{
    complex_multiply, complex_multiply_conjugate, complex_multiply_conjugate_i8,
    complex_multiply_conjugate_i16, complex_multiply_i8, complex_multiply_i16, mix_down,
    mix_down_fixed_i8, mix_down_fixed_i16, mix_up, mix_up_fixed_i8, mix_up_fixed_i16,
};

// Brings NCO / DDS processor into current scope
pub use nco::Nco;

// Brings FIR filter processor into current scope
pub use fir::FirFilter;

// Brings CCSDS Reed-Solomon encoder into current scope
pub use rs_ccsds::{
    CCSDS_RS_CODEWORD_SYMBOLS, CCSDS_RS_DATA_SYMBOLS, CCSDS_RS_PARITY_SYMBOLS,
    ccsds_rs_255_223_encode, ccsds_rs_255_223_parity,
};

pub use q_format::QFormat;

pub use sdr_boundary::{
    complex_f32_to_i8, complex_f32_to_i16, complex_f64_to_i8, complex_f64_to_i16,
    complex_i8_to_f32, complex_i8_to_f64, complex_i16_to_f32, complex_i16_to_f64,
    deinterleave_complex, interleave_complex, rtl_sdr_i8_to_u8, rtl_sdr_u8_to_f32,
    rtl_sdr_u8_to_i8,
};
