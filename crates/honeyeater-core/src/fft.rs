use num_complex::Complex;

pub(crate) mod phastft_impl;
pub(crate) mod rustfft_impl;

/// A trait that abstracts over different FFT backends, allowing for interchangeable implementations.
/// Currently works for PhastFT and RustFFT
pub trait FftWrapper<T> {
    /// Performs a forward Fast Fourier Transform on the provided buffer of complex numbers.
    /// Uses generic type parameter <T> for flexibility
    fn fft(&self, buffer: &mut [Complex<T>]);

    /// Performs a inverse Fast Fourier Transform on the provided buffer of complex numbers.
    /// Uses generic type parameter <T> for flexibility
    fn ifft(&self, buffer: &mut [Complex<T>]);

    /// Returns the size of the FFT that the backend is configured to handle.
    fn size(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::phastft_impl::PhastFftBackend;
    use super::rustfft_impl::RustFftBackend;
    use super::*;
    use honeyeater_test::{assert_close, npy};
    use std::path::PathBuf;

    trait FftScalar: Copy + Default {
        fn from_f64(value: f64) -> Self;
        fn to_f64(self) -> f64;
    }

    impl FftScalar for f32 {
        fn from_f64(value: f64) -> Self {
            value as f32
        }

        fn to_f64(self) -> f64 {
            self as f64
        }
    }

    impl FftScalar for f64 {
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
    fn assert_complex_close<T: FftScalar>(actual: &[Complex<T>], expected: &[Complex<T>]) {
        let (actual_re, actual_im): (Vec<f64>, Vec<f64>) = actual
            .iter()
            .map(|c| (c.re.to_f64(), c.im.to_f64()))
            .unzip();
        let (expected_re, expected_im): (Vec<f64>, Vec<f64>) = expected
            .iter()
            .map(|c| (c.re.to_f64(), c.im.to_f64()))
            .unzip();

        assert_close!(actual_re, expected_re, rtol = 1e-5, atol = 1e-5);
        assert_close!(actual_im, expected_im, rtol = 1e-5, atol = 1e-5);
    }

    // Generic test executor so both backends run through the exact same assertions
    fn run_backend_against_oracle<T: FftScalar>(backend: &dyn FftWrapper<T>, size: usize) {
        let expected_input = load_complex_vector::<T>(&format!("input_{}.npy", size))
            .expect("failed to load complex FFT reference vector");
        let expected_fft = load_complex_vector::<T>(&format!("fft_{}.npy", size))
            .expect("failed to load complex FFT reference vector");
        let expected_ifft = load_complex_vector::<T>(&format!("ifft_{}.npy", size))
            .expect("failed to load complex FFT reference vector");

        let mut buffer = expected_input.clone();

        // Test Forward FFT
        backend.fft(&mut buffer);
        assert_complex_close(&buffer, &expected_fft);

        // Test Inverse FFT
        backend.ifft(&mut buffer);
        assert_complex_close(&buffer, &expected_ifft);
    }

    fn assert_rustfft_matches_oracle<T: FftScalar + rustfft::FftNum>() {
        for size in [8usize, 16, 64] {
            let backend = RustFftBackend::<T>::new(size);
            run_backend_against_oracle(&backend, size);
        }
    }

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
    fn test_phastft_matches_oracle_f64() {
        assert_phastft_matches_oracle::<f64>();
    }

    #[test]
    fn test_phastft_matches_oracle_f32() {
        assert_phastft_matches_oracle::<f32>();
    }
}
