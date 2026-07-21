//Check for symmetric hanning window
pub fn hann_window(n: usize, l: usize) -> f64 {
    hann(n, l, true)
}

//Check for non-periodic hanning window
pub fn hann_window_periodic(n: usize, l: usize) -> f64 {
    hann(n, l, false)
}

pub fn hann(n: usize, l: usize, symmetric: bool) -> f64 {
    debug_assert!(
        n < l,
        "sample index n ({n}) must be less than window length l ({l})"
    );
    match l {
        0 => panic!("Hann window length must be greater than zero"),
        1 => 1.0,
        _ => {
            let denom = if symmetric { l - 1 } else { l };
            let angle = 2.0 * std::f64::consts::PI * n as f64 / denom as f64;
            0.5 - 0.5 * angle.cos()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use honeyeater_test::{assert_close, npy};
    use std::path::PathBuf;

    #[test]
    fn test_hann_window_matches_oracle() {
        let l = 64;

        let mut vector_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        vector_path.push("tests");
        vector_path.push("vectors");
        vector_path.push("hann_64.npy");

        let expected = npy::load_f64(&vector_path);

        let mut actual = Vec::with_capacity(l);
        for n in 0..l {
            actual.push(hann_window(n, l));
        }

        assert_close!(actual, expected, rtol = 1e-12, atol = 1e-15);
    }
}
