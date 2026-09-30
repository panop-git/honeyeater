use num_complex::Complex;
#[cfg(not(feature = "deterministic"))]
use rustfft::FftPlanner;
#[cfg(feature = "deterministic")]
use rustfft::FftPlannerScalar as FftPlanner;
use rustfft::{Fft, FftNum};
use std::cell::RefCell;
use std::sync::Arc;

use super::FftWrapper;

/// A planned complex FFT with reusable scratch space, backed by `RustFFT`.
///
/// Construction allocates plans and scratch storage. Forward and inverse calls
/// allocate no output or scratch buffers. The inverse includes `1/N` scaling.
/// The `deterministic` feature selects scalar algorithms instead of CPU-dependent
/// SIMD dispatch. Floating-point results still use the documented SNR tolerances.
///
/// This processor is `Send` but not `Sync`: scratch storage is owned by one
/// processor. Use a separate processor for each concurrently executing stream.
pub struct RustFftBackend<T: FftNum> {
    forward: Arc<dyn Fft<T>>,
    inverse: Arc<dyn Fft<T>>,
    scratch: RefCell<Vec<Complex<T>>>,
    length: u32,
}

impl<T: FftNum> RustFftBackend<T> {
    /// Plans an FFT of exactly `size` complex samples.
    ///
    /// Arbitrary odd and even lengths are supported. Plans and scratch storage
    /// are allocated once and reused for each transform.
    ///
    /// # Panics
    ///
    /// Panics if `size < 2` or `size > u32::MAX`.
    #[must_use]
    pub fn new(size: usize) -> Self {
        assert!(size >= 2, "FFT requires at least two samples");
        let length = u32::try_from(size).expect("FFT length exceeds u32::MAX");
        let mut planner = FftPlanner::<T>::new();
        let forward = planner.plan_fft_forward(size);
        let inverse = planner.plan_fft_inverse(size);
        let scratch_len = forward
            .get_inplace_scratch_len()
            .max(inverse.get_inplace_scratch_len());
        Self {
            forward,
            inverse,
            scratch: RefCell::new(vec![Complex::new(T::zero(), T::zero()); scratch_len]),
            length,
        }
    }

    /// Returns the spacing between FFT bins, in cycles per sample.
    #[must_use]
    pub fn bin_spacing(&self) -> f64 {
        1.0 / f64::from(self.length)
    }

    /// Returns a bin's signed frequency, in cycles per sample.
    ///
    /// Uses the `SciPy` `fftfreq` ordering: DC, positive frequencies, then negative
    /// frequencies. For even lengths, the Nyquist bin represents `-0.5`.
    ///
    /// # Panics
    ///
    /// Panics if `bin >= size`.
    #[must_use]
    pub fn bin_frequency(&self, bin: usize) -> f64 {
        assert!(
            bin < self.size(),
            "FFT bin must be less than configured size"
        );
        let index = u32::try_from(bin).expect("FFT bin fits within configured u32 length");
        let signed = if index < self.length.div_ceil(2) {
            f64::from(index)
        } else {
            f64::from(index) - f64::from(self.length)
        };
        signed / f64::from(self.length)
    }

    /// Converts a bin frequency to hertz using the caller's sample rate.
    ///
    /// # Panics
    ///
    /// Panics if `bin >= size` or `sample_rate` is not finite and positive.
    #[must_use]
    pub fn bin_frequency_hz(&self, bin: usize, sample_rate: f64) -> f64 {
        assert!(
            sample_rate.is_finite() && sample_rate > 0.0,
            "sample rate must be finite and positive"
        );
        self.bin_frequency(bin) * sample_rate
    }
}

impl<T: FftNum> FftWrapper<T> for RustFftBackend<T> {
    fn fft(&self, buffer: &mut [Complex<T>]) {
        assert_eq!(
            buffer.len(),
            self.size(),
            "buffer length must equal configured FFT size"
        );
        self.forward
            .process_with_scratch(buffer, &mut self.scratch.borrow_mut());
    }

    fn ifft(&self, buffer: &mut [Complex<T>]) {
        assert_eq!(
            buffer.len(),
            self.size(),
            "buffer length must equal configured FFT size"
        );
        self.inverse
            .process_with_scratch(buffer, &mut self.scratch.borrow_mut());
        let n_inv = T::from_f64(1.0 / f64::from(self.length))
            .expect("floating-point FFT scalar can represent inverse length");
        for value in buffer {
            *value = *value * n_inv;
        }
    }

    fn size(&self) -> usize {
        self.length as usize
    }
}
