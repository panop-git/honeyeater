/// Computes the symmetric Hann window value at sample index `n` for a window of length `l`.
/// This variant is designed for non-periodic signals only. The symmetric value is set to true.
pub fn hann_window(n: usize, l: usize) -> f64 {
    hann(n, l, true)
}

/// Computes the symmetric Hann window value at sample index `n` for a window of length `l`.
/// This variant is designed for periodic signals only. The symmetric value is set to false.
pub fn hann_window_periodic(n: usize, l: usize) -> f64 {
    hann(n, l, false)
}

/// Internal crate function for calculating Hann window values.
///
/// This function implements the discrete-time Hann window formula:
/// omega(n) = 0.5 - 0.5 * cos((2 * pi * n)/D), where D depends on the symmetry flag.
/// For symmetry = 'true', D = l - 1;
/// For symmetry = 'false', D = l.
///
/// Integrated panic and assert macros to ensure input parameters are within valid bounds
pub(crate) fn hann(n: usize, l: usize, symmetric: bool) -> f64 {
    // Ensures that the sample index n is less than the window length l
    assert!(
        n < l,
        "sample index n ({n}) must be less than window length l ({l})"
    );

    // Condition checks
    match l {
        0 => panic!("Hann window length must be greater than zero"),
        1 => 1.0,

        // For lengths 2 or greater
        _ => {
            let denom = if symmetric { l - 1 } else { l }; // Changes condition for periodic vs symmetric
            let angle = 2.0 * std::f64::consts::PI * n as f64 / denom as f64; // Uses discrete-time formula for Hann window
            0.5 - 0.5 * angle.cos()
        }
    }
}

#[cfg(test)]
// Isolates testing from rest of code
mod tests {
    use super::*; // Imports everything from parent module
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

        let expected = npy::load_f64(&vector_path);

        // Appends hann_window function outputs to vector
        let mut actual = Vec::with_capacity(l); // Creates empty vector with length l
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

        let expected = npy::load_f64(&vector_path);

        let mut actual = Vec::with_capacity(l);
        for n in 0..l {
            actual.push(hann_window_periodic(n, l));
        }

        assert_close!(actual, expected, rtol = 1e-12, atol = 1e-15);
    }
}
