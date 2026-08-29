use num_complex::{Complex, ComplexFloat};

// Floating-Point Mixer Primitives
/// Mixes the signal up in frequency by multiplying it with the NCO output.
/// Formula: y = signal * nco
///
/// Works with any type T that implements ComplexFloat (e.g., f32, f64).
pub fn mix_up<T: ComplexFloat>(signal: Complex<T>, nco: Complex<T>) -> Complex<T> {
    signal * nco
}

/// Mixes the signal down in frequency by multiplying it with the complex conjugate
/// of the NCO output (the mixer primitive).
/// Formula: y = signal * nco*
///
/// Works with any type T that implements ComplexFloat (e.g., f32, f64).
pub fn mix_down<T: ComplexFloat>(signal: Complex<T>, nco: Complex<T>) -> Complex<T> {
    signal * nco.conj()
}

// Fixed-Point Mixer Primitives
/// Mixes the signal up in frequency in fixed-point (i32) with half-up rounding.
/// Formula: y = signal * nco
///
/// # Arguments
/// * signal - The complex signal as a pair of i32 values.
/// * nco - The complex NCO output as a pair of i32 values.
/// * shift - The number of bits to shift the result right to maintain scale.
pub fn mix_up_fixed(signal: Complex<i32>, nco: Complex<i32>, shift: u32) -> Complex<i32> {
    let round = if shift > 0 { 1_i64 << (shift - 1) } else { 0 };
    let re =
        ((signal.re as i64 * nco.re as i64) - (signal.im as i64 * nco.im as i64) + round) >> shift;
    let im =
        ((signal.re as i64 * nco.im as i64) + (signal.im as i64 * nco.re as i64) + round) >> shift;
    Complex::new(re as i32, im as i32)
}

/// Mixes the signal down in frequency in fixed-point (i32) with half-up rounding.
/// Formula: y = signal * nco*
///
/// # Arguments
/// * signal - The complex signal as a pair of i32 values.
/// * nco - The complex NCO output as a pair of i32 values.
/// * shift - The number of bits to shift the result right to maintain scale.
pub fn mix_down_fixed(signal: Complex<i32>, nco: Complex<i32>, shift: u32) -> Complex<i32> {
    let round = if shift > 0 { 1_i64 << (shift - 1) } else { 0 };
    let re =
        ((signal.re as i64 * nco.re as i64) + (signal.im as i64 * nco.im as i64) + round) >> shift;
    let im =
        ((signal.im as i64 * nco.re as i64) - (signal.re as i64 * nco.im as i64) + round) >> shift;
    Complex::new(re as i32, im as i32)
}

#[cfg(test)]
mod tests {
    use super::*;
    use honeyeater_test::npy;
    use std::path::Path;

    fn load_complex_vec_f64(filename: &str) -> npy::NpyResult<Vec<Complex<f64>>> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("vectors")
            .join("mixer")
            .join(filename);
        npy::load_complex_f64(&path)
    }

    fn load_complex_vec_i32(filename: &str) -> npy::NpyResult<Vec<Complex<i32>>> {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("vectors")
            .join("mixer")
            .join(filename);
        npy::load_complex_i32(&path)
    }

    #[test]
    fn test_mix_up_float() {
        let vec_lengths = [8, 16, 64];
        for n in vec_lengths {
            let signal = load_complex_vec_f64(&format!("input_{}.npy", n))
                .expect("failed to load complex mixer input vector");
            let nco = load_complex_vec_f64(&format!("nco_{}.npy", n))
                .expect("failed to load complex mixer NCO vector");
            let expected = load_complex_vec_f64(&format!("mix_up_{}.npy", n))
                .expect("failed to load complex mixer reference vector");

            let result: Vec<Complex<f64>> = signal
                .iter()
                .zip(nco.iter())
                .map(|(&s, &nc)| mix_up(s, nc))
                .collect();

            for (r, e) in result.iter().zip(expected.iter()) {
                assert!((r.re - e.re).abs() < 1e-9);
                assert!((r.im - e.im).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn test_mix_down_float() {
        let vec_lengths = [8, 16, 64];
        for n in vec_lengths {
            let signal = load_complex_vec_f64(&format!("input_{}.npy", n))
                .expect("failed to load complex mixer input vector");
            let nco = load_complex_vec_f64(&format!("nco_{}.npy", n))
                .expect("failed to load complex mixer NCO vector");
            let expected = load_complex_vec_f64(&format!("mix_down_{}.npy", n))
                .expect("failed to load complex mixer reference vector");

            let result: Vec<Complex<f64>> = signal
                .iter()
                .zip(nco.iter())
                .map(|(&s, &nc)| mix_down(s, nc))
                .collect();

            for (r, e) in result.iter().zip(expected.iter()) {
                assert!((r.re - e.re).abs() < 1e-9);
                assert!((r.im - e.im).abs() < 1e-9);
            }
        }
    }

    #[test]
    fn test_mix_up_fixed() {
        let vec_lengths = [8, 16, 64];
        let shift = 10;
        for n in vec_lengths {
            let signal = load_complex_vec_i32(&format!("input_fixed_{}.npy", n))
                .expect("failed to load complex mixer input vector");
            let nco = load_complex_vec_i32(&format!("nco_fixed_{}.npy", n))
                .expect("failed to load complex mixer NCO vector");
            let expected = load_complex_vec_i32(&format!("mix_up_fixed_{}.npy", n))
                .expect("failed to load complex mixer reference vector");

            let result: Vec<Complex<i32>> = signal
                .iter()
                .zip(nco.iter())
                .map(|(&s, &nc)| mix_up_fixed(s, nc, shift))
                .collect();

            for (r, e) in result.iter().zip(expected.iter()) {
                assert_eq!(r.re, e.re);
                assert_eq!(r.im, e.im);
            }
        }
    }

    #[test]
    fn test_mix_down_fixed() {
        let vec_lengths = [8, 16, 64];
        let shift = 10;
        for n in vec_lengths {
            let signal = load_complex_vec_i32(&format!("input_fixed_{}.npy", n))
                .expect("failed to load complex mixer input vector");
            let nco = load_complex_vec_i32(&format!("nco_fixed_{}.npy", n))
                .expect("failed to load complex mixer NCO vector");
            let expected = load_complex_vec_i32(&format!("mix_down_fixed_{}.npy", n))
                .expect("failed to load complex mixer reference vector");

            let result: Vec<Complex<i32>> = signal
                .iter()
                .zip(nco.iter())
                .map(|(&s, &nc)| mix_down_fixed(s, nc, shift))
                .collect();

            for (r, e) in result.iter().zip(expected.iter()) {
                assert_eq!(r.re, e.re);
                assert_eq!(r.im, e.im);
            }
        }
    }
}
