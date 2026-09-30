//! # honeyeater
//!
//! Digital signal processing primitives for radio-frequency and electrical
//! signals, operating on CPU-side sample streams from radios, digitisers,
//! and simulators.
//!
//! This facade re-exports the public API of [`honeyeater_core`], including
//! windows, [`Biquad`], [`FirFilter`], [`Nco`], complex multiply and mixers,
//! [`q_format`] and SDR sample conversions, CRCs, and the CCSDS RS(255,223)
//! encoder. Depend on this crate to use those primitives through one import path.
//!
//! ## Status
//!
//! Version 0.0.1 is prepared for the first kernel release. The API is still
//! experimental. [`FftWrapper`] exposes the FFT backend contract; the current
//! `PhastFT` and `RustFFT` implementations are compiled only for core tests, so
//! this release preparation does not yet provide a public FFT constructor.
//! See the repository `docs/roadmap.md` for the remaining release gates.
//!
//! ## Generate and filter a complex tone
//!
//! ```
//! use honeyeater::{Complex, FirFilter, Nco};
//!
//! let mut oscillator = Nco::<Complex<f32>>::new(0.125);
//! let mut input = [Complex::new(0.0_f32, 0.0); 16];
//! oscillator.fill(&mut input);
//!
//! let taps = [Complex::new(0.5_f32, 0.0); 2];
//! let mut filter = FirFilter::<Complex<f32>>::new(&taps);
//! let mut output = [Complex::new(0.0_f32, 0.0); 16];
//! filter.process(&input, &mut output);
//! ```
//!
//! ## Encode a CCSDS information block
//!
//! ```
//! use honeyeater::{CCSDS_RS_DATA_SYMBOLS, ccsds_rs_255_223_encode, crc32_castagnoli};
//!
//! let information = [0x55_u8; CCSDS_RS_DATA_SYMBOLS];
//! let codeword = ccsds_rs_255_223_encode(&information);
//! assert_eq!(&codeword[..CCSDS_RS_DATA_SYMBOLS], &information);
//! assert_eq!(crc32_castagnoli(b"123456789"), 0xe306_9283);
//! ```
//!
//! ## Licence
//!
//! Dual-licensed under MIT or Apache-2.0, at your option.

#![forbid(unsafe_code)]

pub use honeyeater_core::*;
