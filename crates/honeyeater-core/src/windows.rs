pub trait WindowValue: Copy + Default {
    fn from_f64(value: f64) -> Self;
}

impl WindowValue for f32 {
    fn from_f64(value: f64) -> Self {
        value as f32
    }
}

impl WindowValue for f64 {
    fn from_f64(value: f64) -> Self {
        value
    }
}

// Hann window module
pub(crate) mod hann {
    use super::WindowValue;

    /// Computes the symmetric Hann window value at sample index `n` for a window of length `l`.
    /// This variant is designed for non-periodic signals only. The symmetric value is set to true.
    pub fn hann_window<T: WindowValue>(n: usize, l: usize) -> T {
        hann::<T>(n, l, true)
    }

    /// Computes the periodic Hann window value at sample index `n` for a window of length `l`.
    /// This variant is designed for periodic signals only. The symmetric value is set to false.
    pub fn hann_window_periodic<T: WindowValue>(n: usize, l: usize) -> T {
        hann::<T>(n, l, false)
    }

    /// Internal crate function for calculating Hann window values.
    ///
    /// This function implements the discrete-time Hann window formula:
    /// omega(n) = 0.5 - 0.5 * cos((2 * pi * n)/D), where D depends on the symmetry flag.
    /// For symmetry = 'true', D = l - 1;
    /// For symmetry = 'false', D = l.
    ///
    /// Integrated panic and assert macros to ensure input parameters are within valid bounds
    pub(crate) fn hann<T: WindowValue>(n: usize, l: usize, symmetric: bool) -> T {
        // Ensures that the sample index n is less than the window length l
        assert!(
            n < l,
            "sample index n ({n}) must be less than window length l ({l})"
        );

        // Condition checks
        match l {
            0 => panic!("Hann window length must be greater than zero"),
            1 => T::from_f64(1.0),

            // For lengths 2 or greater
            _ => {
                let denom = if symmetric { l - 1 } else { l }; // Changes condition for periodic vs symmetric
                let angle = 2.0 * std::f64::consts::PI * n as f64 / denom as f64; // Uses discrete-time formula for Hann window
                T::from_f64(0.5 - 0.5 * angle.cos())
            }
        }
    }
}

// Hamming window module
pub(crate) mod hamming {
    use super::WindowValue;

    /// Computes the symmetric Hamming window value at sample index `n` for a window of length `l`.
    /// This variant is designed for non-periodic signals only. The symmetric value is set to true.
    pub fn hamming_window<T: WindowValue>(n: usize, l: usize) -> T {
        hamming::<T>(n, l, true)
    }

    /// Computes the periodic Hamming window value at sample index `n` for a window of length `l`.
    /// This variant is designed for periodic signals only. The symmetric value is set to false.
    pub fn hamming_window_periodic<T: WindowValue>(n: usize, l: usize) -> T {
        hamming::<T>(n, l, false)
    }

    /// Internal crate function for calculating Hamming window values.
    ///
    /// This function implements the discrete-time Hamming window formula:
    /// omega(n) = 0.54 - 0.46 * cos((2 * pi * n)/D), where D depends on the symmetry flag.
    /// For symmetry = 'true', D = l - 1;
    /// For symmetry = 'false', D = l.
    ///
    /// Integrated panic and assert macros to ensure input parameters are within valid bounds
    pub(crate) fn hamming<T: WindowValue>(n: usize, l: usize, symmetric: bool) -> T {
        // Ensures that the sample index n is less than the window length l
        assert!(
            n < l,
            "sample index n ({n}) must be less than window length l ({l})"
        );

        // Condition checks
        match l {
            0 => panic!("Hamming window length must be greater than zero"),
            1 => T::from_f64(1.0),

            // For lengths 2 or greater
            _ => {
                let denom = if symmetric { l - 1 } else { l }; // Changes condition for periodic vs symmetric
                let angle = 2.0 * std::f64::consts::PI * n as f64 / denom as f64; // Uses discrete-time formula for Hamming window
                T::from_f64(0.54 - 0.46 * angle.cos())
            }
        }
    }
}

pub(crate) mod blackmanharris {
    use super::WindowValue;

    /// Computes the symmetric Blackman-Harris window value at sample index `n` for a window of length `l`.
    /// This variant is designed for non-periodic signals only. The symmetric value is set to true.
    pub fn blackmanharris_window<T: WindowValue>(n: usize, l: usize) -> T {
        blackmanharris::<T>(n, l, true)
    }

    /// Computes the periodic Blackman-Harris window value at sample index `n` for a window of length `l`.
    /// This variant is designed for periodic signals only. The symmetric value is set to false.
    pub fn blackmanharris_window_periodic<T: WindowValue>(n: usize, l: usize) -> T {
        blackmanharris::<T>(n, l, false)
    }

    /// Internal crate function for calculating Blackman-Harris window values.
    ///
    /// This function implements the discrete-time Blackman-Harris window formula:Hann
    /// omega(n) = 0.35875 - 0.48829 * cos((2 * pi * n)/D) + 0.14128 * cos((4 * pi * n)/D) - 0.01168 * cos((6 * pi * n)/D), where D depends on the symmetry flag.
    /// For symmetry = 'true', D = l - 1;
    /// For symmetry = 'false', D = l.
    ///
    /// Integrated panic and assert macros to ensure input parameters are within valid bounds
    pub(crate) fn blackmanharris<T: WindowValue>(n: usize, l: usize, symmetric: bool) -> T {
        // Ensures that the sample index n is less than the window length l
        assert!(
            n < l,
            "sample index n ({n}) must be less than window length l ({l})"
        );

        // Condition checks
        match l {
            0 => panic!("Blackman-Harris window length must be greater than zero"),
            1 => T::from_f64(1.0),

            // For lengths 2 or greater
            _ => {
                let denom = if symmetric { l - 1 } else { l }; // Changes condition for periodic vs symmetric
                let angle = 2.0 * std::f64::consts::PI * n as f64 / denom as f64; // Uses discrete-time formula for Blackman-Harris window
                T::from_f64(
                    0.35875 - 0.48829 * angle.cos() + 0.14128 * (2.0 * angle).cos()
                        - 0.01168 * (3.0 * angle).cos(),
                )
            }
        }
    }
}

pub(crate) mod kaiser {
    use super::WindowValue;

    /// Computes the symmetric Kaiser window value at sample index `n` for a window of length `l`.
    /// This variant is designed for non-periodic signals only. The symmetric value is set to true.
    pub fn kaiser_window<T: WindowValue>(n: usize, l: usize, beta: f64) -> T {
        kaiser::<T>(n, l, beta, true)
    }

    /// Computes the periodic Kaiser window value at sample index `n` for a window of length `l`.
    /// This variant is designed for periodic signals only. The symmetric value is set to false.
    pub fn kaiser_window_periodic<T: WindowValue>(n: usize, l: usize, beta: f64) -> T {
        kaiser::<T>(n, l, beta, false)
    }

    /// Zero-order modified Bessel function of the first kind can be defined by the following:
    /// I_0(x) = sum(0 -> infty, (((x/2)^k)/k!)^2)
    fn bessel_i0(x: f64) -> f64 {
        let mut sum = 1.0;
        let mut term = 1.0;
        // k only loops to 30 for reduced computation
        for k in 1..=30 {
            term *= (x * x / 4.0) / ((k as f64) * (k as f64));
            sum += term;
        }
        sum
    }

    /// Internal crate function for calculating Kaiser window values.
    ///
    /// This function implements the discrete-time Kaiser window formula:
    /// omega(n) = I_0 * (beta * sqrt(1 - x^2)) / I_0(beta), where x = 2n/D - 1, where D depends on the symmetry flag.
    /// For symmetry = 'true', D = l - 1;
    /// For symmetry = 'false', D = l.
    ///
    /// Integrated panic and assert macros to ensure input parameters are within valid bounds
    pub(crate) fn kaiser<T: WindowValue>(n: usize, l: usize, beta: f64, symmetric: bool) -> T {
        // Ensures that the sample index n is less than the window length l
        assert!(
            n < l,
            "sample index n ({n}) must be less than window length l ({l})"
        );

        // Condition checks
        match l {
            0 => panic!("Kaiser window length must be greater than zero"),
            1 => T::from_f64(1.0),

            _ => {
                let denom = if symmetric { l - 1 } else { l };
                let x = (2.0 * n as f64 / denom as f64) - 1.0;
                let sqrt_term = if x.abs() >= 1.0 {
                    0.0
                } else {
                    (1.0 - x * x).sqrt()
                };
                let numerator = bessel_i0(beta * sqrt_term);
                let denominator = bessel_i0(beta);
                T::from_f64(numerator / denominator)
            }
        }
    }
}

#[cfg(test)]
// Isolates testing from rest of code
mod tests {
    mod hann_tests {
        use super::super::hann::*; // Imports everything from parent module
        use honeyeater_test::{assert_close, npy}; // Imports required tools
        use std::path::PathBuf;

        // Test f64 symmetric Hann window against .npy reference vector
        #[test] // Executed when cargo test is run
        fn test_hann_window_matches_oracle() {
            let l = 64;

            // Builds path towards npy reference vectors
            let mut vector_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            vector_path.push("tests");
            vector_path.push("vectors");
            vector_path.push("windows");
            vector_path.push("hann");
            vector_path.push("hann_64.npy");

            let expected =
                npy::load_f64(&vector_path).expect("failed to load window reference vector");

            // Appends hann_window function outputs to vector
            let mut actual: Vec<f64> = Vec::with_capacity(l); // Creates empty vector with length l
            for n in 0..l {
                actual.push(hann_window(n, l));
            }

            // Compares actual and expected vectors with specified tolerances
            assert_close!(actual, expected, rtol = 1e-12, atol = 1e-15);
        }

        // Test f64 periodic Hann window against .npy reference vector
        #[test]
        fn test_hann_window_periodic_matches_oracle() {
            let l = 64;

            let mut vector_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            vector_path.push("tests");
            vector_path.push("vectors");
            vector_path.push("windows");
            vector_path.push("hann");
            vector_path.push("hann_periodic_64.npy");

            let expected =
                npy::load_f64(&vector_path).expect("failed to load window reference vector");

            let mut actual: Vec<f64> = Vec::with_capacity(l);
            for n in 0..l {
                actual.push(hann_window_periodic(n, l));
            }

            assert_close!(actual, expected, rtol = 1e-12, atol = 1e-15);
        }
    }

    mod hamming_tests {
        use super::super::hamming::*; // Imports everything from parent module
        use honeyeater_test::{assert_close, npy}; // Imports required tools
        use std::path::PathBuf;

        // Test f64 symmetric Hamming window against .npy reference vector
        #[test] // Executed when cargo test is run
        fn test_hamming_window_matches_oracle() {
            let l = 64;

            // Builds path towards npy reference vectors
            let mut vector_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            vector_path.push("tests");
            vector_path.push("vectors");
            vector_path.push("windows");
            vector_path.push("hamming");
            vector_path.push("hamming_64.npy");

            let expected =
                npy::load_f64(&vector_path).expect("failed to load window reference vector");

            // Appends hamming_window function outputs to vector
            let mut actual: Vec<f64> = Vec::with_capacity(l); // Creates empty vector with length l
            for n in 0..l {
                actual.push(hamming_window(n, l));
            }

            // Compares actual and expected vectors with specified tolerances
            assert_close!(actual, expected, rtol = 1e-12, atol = 1e-15);
        }

        // Test f64 periodic Hamming window against .npy reference vector
        #[test] // Executed when cargo test is run
        fn test_hamming_window_periodic_matches_oracle() {
            let l = 64;

            // Builds path towards npy reference vectors
            let mut vector_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            vector_path.push("tests");
            vector_path.push("vectors");
            vector_path.push("windows");
            vector_path.push("hamming");
            vector_path.push("hamming_periodic_64.npy");

            let expected =
                npy::load_f64(&vector_path).expect("failed to load window reference vector");

            // Appends hamming_window function outputs to vector
            let mut actual: Vec<f64> = Vec::with_capacity(l); // Creates empty vector with length l
            for n in 0..l {
                actual.push(hamming_window_periodic(n, l));
            }

            // Compares actual and expected vectors with specified tolerances
            assert_close!(actual, expected, rtol = 1e-12, atol = 1e-15);
        }
    }

    mod blackmanharris_tests {
        use super::super::blackmanharris::*; // Imports everything from parent module
        use honeyeater_test::{assert_close, npy}; // Imports required tools
        use std::path::PathBuf;

        // Test f64 symmetric Blackman-Harris window against .npy reference vector
        #[test] // Executed when cargo test is run
        fn test_blackmanharris_window_matches_oracle() {
            let l = 64;

            // Builds path towards npy reference vectors
            let mut vector_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            vector_path.push("tests");
            vector_path.push("vectors");
            vector_path.push("windows");
            vector_path.push("blackmanharris");
            vector_path.push("blackmanharris_64.npy");

            let expected =
                npy::load_f64(&vector_path).expect("failed to load window reference vector");

            // Appends blackmanharris_window function outputs to vector
            let mut actual: Vec<f64> = Vec::with_capacity(l); // Creates empty vector with length l
            for n in 0..l {
                actual.push(blackmanharris_window(n, l));
            }

            // Compares actual and expected vectors with specified tolerances
            assert_close!(actual, expected, rtol = 1e-12, atol = 1e-15);
        }

        // Test f64 periodic Blackman-Harris window against .npy reference vector
        #[test] // Executed when cargo test is run
        fn test_blackmanharris_window_periodic_matches_oracle() {
            let l = 64;

            // Builds path towards npy reference vectors
            let mut vector_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            vector_path.push("tests");
            vector_path.push("vectors");
            vector_path.push("windows");
            vector_path.push("blackmanharris");
            vector_path.push("blackmanharris_periodic_64.npy");

            let expected =
                npy::load_f64(&vector_path).expect("failed to load window reference vector");

            // Appends blackmanharris_window function outputs to vector
            let mut actual: Vec<f64> = Vec::with_capacity(l); // Creates empty vector with length l
            for n in 0..l {
                actual.push(blackmanharris_window_periodic(n, l));
            }

            // Compares actual and expected vectors with specified tolerances
            assert_close!(actual, expected, rtol = 1e-12, atol = 1e-15);
        }
    }

    mod kaiser_tests {
        use super::super::kaiser::*; // Imports everything from parent module
        use honeyeater_test::{assert_close, npy}; // Imports required tools
        use std::path::PathBuf;

        // Test f64 symmetric Kaiser window against .npy reference vector
        #[test] // Executed when cargo test is run
        fn test_kaiser_window_matches_oracle() {
            let l = 64;
            let beta = 14.0;

            // Builds path towards npy reference vectors
            let mut vector_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            vector_path.push("tests");
            vector_path.push("vectors");
            vector_path.push("windows");
            vector_path.push("kaiser");
            vector_path.push("kaiser_64.npy");

            let expected =
                npy::load_f64(&vector_path).expect("failed to load window reference vector");

            // Appends kaiser_window function outputs to vector
            let mut actual: Vec<f64> = Vec::with_capacity(l); // Creates empty vector with length l
            for n in 0..l {
                actual.push(kaiser_window(n, l, beta));
            }

            // Compares actual and expected vectors with specified tolerances
            assert_close!(actual, expected, rtol = 1e-12, atol = 1e-15);
        }

        // Test f64 periodic Kaiser window against .npy reference vector
        #[test] // Executed when cargo test is run
        fn test_kaiser_window_periodic_matches_oracle() {
            let l = 64;
            let beta = 14.0;

            // Builds path towards npy reference vectors
            let mut vector_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
            vector_path.push("tests");
            vector_path.push("vectors");
            vector_path.push("windows");
            vector_path.push("kaiser");
            vector_path.push("kaiser_periodic_64.npy");

            let expected =
                npy::load_f64(&vector_path).expect("failed to load window reference vector");

            // Appends kaiser_window function outputs to vector
            let mut actual: Vec<f64> = Vec::with_capacity(l); // Creates empty vector with length l
            for n in 0..l {
                actual.push(kaiser_window_periodic(n, l, beta));
            }

            // Compares actual and expected vectors with specified tolerances
            assert_close!(actual, expected, rtol = 1e-12, atol = 1e-15);
        }
    }
}
