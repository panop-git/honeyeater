use num_complex::Complex;

pub mod phastft_impl;
pub mod rustfft_impl;

pub trait FftWrapper<T> {
    fn fft(&self, buffer: &mut [Complex<T>]);
    fn ifft(&self, buffer: &mut [Complex<T>]);
    fn size(&self) -> usize;
}

#[cfg(test)]
mod tests {
    use super::phastft_impl::PhastFftBackend;
    use super::rustfft_impl::RustFftBackend;
    use super::*;
    use honeyeater_test::{assert_close, npy};
    use std::path::PathBuf;

    // Helper to streamline loading multiple .npy files
    fn load_complex_vector(filename: &str) -> Vec<Complex<f64>> {
        let mut vector_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        vector_path.push("tests");
        vector_path.push("vectors");
        vector_path.push("fft");
        vector_path.push(filename);

        npy::load_complex_f64(&vector_path)
    }

    // Helper to test Complex vectors using a macro designed for real numbers
    fn assert_complex_close(actual: &[Complex<f64>], expected: &[Complex<f64>]) {
        // Unzip the complex numbers into separate real and imaginary vectors
        let (actual_re, actual_im): (Vec<f64>, Vec<f64>) =
            actual.iter().map(|c| (c.re, c.im)).unzip();
        let (expected_re, expected_im): (Vec<f64>, Vec<f64>) =
            expected.iter().map(|c| (c.re, c.im)).unzip();

        // Run assert_close! on both components independently
        assert_close!(actual_re, expected_re, rtol = 1e-5, atol = 1e-5);
        assert_close!(actual_im, expected_im, rtol = 1e-5, atol = 1e-5);
    }

    // Generic test executor so both backends run through the exact same assertions
    fn run_backend_against_oracle(backend: &dyn FftWrapper<f64>, size: usize) {
        let expected_input = load_complex_vector(&format!("input_{}.npy", size));
        let expected_fft = load_complex_vector(&format!("fft_{}.npy", size));
        let expected_ifft = load_complex_vector(&format!("ifft_{}.npy", size));

        let mut buffer = expected_input.clone();

        // Test Forward FFT
        backend.fft(&mut buffer);
        assert_complex_close(&buffer, &expected_fft);

        // Test Inverse FFT
        backend.ifft(&mut buffer);
        assert_complex_close(&buffer, &expected_ifft);
    }

    #[test]
    fn test_rustfft_matches_oracle() {
        let size = 64;
        let backend = RustFftBackend::<f64>::new(size);
        run_backend_against_oracle(&backend, size);
    }

    #[test]
    fn test_phastft_matches_oracle() {
        let size = 64;
        let backend = PhastFftBackend::<f64>::new(size);
        run_backend_against_oracle(&backend, size);
    }
}
