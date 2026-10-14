use num_complex::Complex;

#[cfg(all(test, not(feature = "deterministic")))]
pub(crate) mod phastft_impl;
mod rustfft_impl;

pub use rustfft_impl::RustFftBackend;

/// A trait that abstracts over different FFT backends, allowing for interchangeable implementations.
/// Currently works for `PhastFT` and `RustFFT`
pub trait FftWrapper<T> {
    /// Performs a forward Fast Fourier Transform on the provided buffer of complex numbers.
    ///
    /// # Panics
    ///
    /// Panics if the buffer length differs from the configured transform size.
    fn fft(&self, buffer: &mut [Complex<T>]);

    /// Performs a inverse Fast Fourier Transform on the provided buffer of complex numbers.
    ///
    /// # Panics
    ///
    /// Panics if the buffer length differs from the configured transform size.
    fn ifft(&self, buffer: &mut [Complex<T>]);

    /// Returns the size of the FFT that the backend is configured to handle.
    fn size(&self) -> usize;
}

#[cfg(test)]
mod tests {
    #[cfg(not(feature = "deterministic"))]
    use super::phastft_impl::PhastFftBackend;
    use super::rustfft_impl::RustFftBackend;
    use super::*;
    use honeyeater_test::{assert_snr_db, npy};
    use std::path::PathBuf;

    trait FftScalar: Copy + Default {
        const MIN_SNR_DB: f64;
        fn from_f64(value: f64) -> Self;
        fn to_f64(self) -> f64;
    }

    #[allow(clippy::cast_possible_truncation)]
    impl FftScalar for f32 {
        const MIN_SNR_DB: f64 = 60.0;
        fn from_f64(value: f64) -> Self {
            value as f32
        }

        fn to_f64(self) -> f64 {
            f64::from(self)
        }
    }

    impl FftScalar for f64 {
        const MIN_SNR_DB: f64 = 120.0;
        fn from_f64(value: f64) -> Self {
            value
        }

        fn to_f64(self) -> f64 {
            self
        }
    }

    // Helper to streamline loading multiple .npy files
    fn load_complex_vector<T: FftScalar>(filename: &str) -> npy::NpyResult<Vec<Complex<T>>> {
        let mut vector_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        vector_path.push("tests");
        vector_path.push("vectors");
        vector_path.push("fft");
        vector_path.push(filename);

        let values = npy::load_complex_f64(&vector_path)?;

        Ok(values
            .into_iter()
            .map(|value| Complex::new(T::from_f64(value.re), T::from_f64(value.im)))
            .collect())
    }

    // Helper to test Complex vectors using a macro designed for real numbers
    fn assert_complex_snr<T: FftScalar>(actual: &[Complex<T>], expected: &[Complex<T>]) {
        let actual_flat: Vec<f64> = actual
            .iter()
            .flat_map(|c| [c.re.to_f64(), c.im.to_f64()])
            .collect();

        let expected_flat: Vec<f64> = expected
            .iter()
            .flat_map(|c| [c.re.to_f64(), c.im.to_f64()])
            .collect();

        assert_snr_db!(actual_flat, expected_flat, min_db = T::MIN_SNR_DB);
    }

    // Generic test executor so both backends run through the exact same assertions
    fn run_backend_against_oracle<T: FftScalar>(backend: &dyn FftWrapper<T>, size: usize) {
        let expected_input = load_complex_vector::<T>(&format!("input_{size}.npy"))
            .expect("failed to load complex FFT reference vector");
        let expected_fft = load_complex_vector::<T>(&format!("fft_{size}.npy"))
            .expect("failed to load complex FFT reference vector");
        let expected_inverse = load_complex_vector::<T>(&format!("ifft_{size}.npy"))
            .expect("failed to load complex FFT reference vector");

        let mut buffer = expected_input.clone();

        // Test Forward FFT
        backend.fft(&mut buffer);
        assert_complex_snr(&buffer, &expected_fft);

        // Test Inverse FFT
        backend.ifft(&mut buffer);
        assert_complex_snr(&buffer, &expected_inverse);
    }

    fn assert_rustfft_matches_oracle<T: FftScalar + rustfft::FftNum>() {
        for size in [8usize, 16, 64] {
            let backend = RustFftBackend::<T>::new(size);
            run_backend_against_oracle(&backend, size);
        }
    }

    #[cfg(not(feature = "deterministic"))]
    fn assert_phastft_matches_oracle<T: FftScalar + phastft_impl::PhastFftFloat>() {
        for size in [8usize, 16, 64] {
            let backend = PhastFftBackend::<T>::new(size);
            run_backend_against_oracle(&backend, size);
        }
    }

    #[test]
    fn test_rustfft_matches_oracle_f64() {
        assert_rustfft_matches_oracle::<f64>();
    }

    #[test]
    fn test_rustfft_matches_oracle_f32() {
        assert_rustfft_matches_oracle::<f32>();
    }

    #[test]
    #[cfg(not(feature = "deterministic"))]
    fn test_phastft_matches_oracle_f64() {
        assert_phastft_matches_oracle::<f64>();
    }

    #[test]
    #[cfg(not(feature = "deterministic"))]
    fn test_phastft_matches_oracle_f32() {
        assert_phastft_matches_oracle::<f32>();
    }
}
