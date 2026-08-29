use std::ops::{Add, Mul, Neg, Sub};

pub trait BiquadScalar:
    Copy + Default + Add<Output = Self> + Sub<Output = Self> + Mul<Output = Self> + Neg<Output = Self>
{
    fn from_f64(value: f64) -> Self;
}

impl BiquadScalar for f32 {
    fn from_f64(value: f64) -> Self {
        value as f32
    }
}

impl BiquadScalar for f64 {
    fn from_f64(value: f64) -> Self {
        value
    }
}

/// A second-order biquad filter state machine.
///
/// The filter is parameterised by its coefficients and the delay-line state
/// carried between samples. The implementation is generic over floating-point
/// sample types so the same filter core can be used for `f32` or `f64`.
pub struct Biquad<T: BiquadScalar> {
    a: [T; 3],
    b: [T; 3],
    x: [T; 2],
    y: [T; 2],
}

impl<T: BiquadScalar> Biquad<T> {
    /// Establish coefficients for lowpass filter
    pub fn lowpass(sample_rate: f64, cutoff_freq: f64, q: f64) -> Self {
        let w0 = 2.0 * std::f64::consts::PI * cutoff_freq / sample_rate;
        let alpha = w0.sin() / (2.0 * q);
        let b0 = (1.0 - w0.cos()) / 2.0;
        let b1 = 1.0 - w0.cos();
        let b2 = (1.0 - w0.cos()) / 2.0;
        let a0 = 1.0 + alpha;
        let a1 = -2.0 * w0.cos();
        let a2 = 1.0 - alpha;

        // Pre-calculate coefficients divided by a0 for efficiency
        Self {
            b: [
                T::from_f64(b0 / a0),
                T::from_f64(b1 / a0),
                T::from_f64(b2 / a0),
            ],
            a: [T::from_f64(1.0), T::from_f64(a1 / a0), T::from_f64(a2 / a0)],
            x: [T::from_f64(0.0), T::from_f64(0.0)], // x[n-1], x[n-2]
            y: [T::from_f64(0.0), T::from_f64(0.0)], // y[n-1], y[n-2]
        }
    }

    /// Process one sample through the biquad filter.
    pub fn biquad_filter(&mut self, input: T) -> T {
        let output = (self.b[0] * input) + (self.b[1] * self.x[0]) + (self.b[2] * self.x[1])
            - (self.a[1] * self.y[0])
            - (self.a[2] * self.y[1]);

        // Iterate through samples
        self.x[1] = self.x[0];
        self.x[0] = input;
        self.y[1] = self.y[0];
        self.y[0] = output;

        output
    }
}

#[cfg(test)]
mod biquad_tests {
    use super::*; // Imports Biquad from the parent module
    use honeyeater_test::{assert_close, npy};
    use std::path::PathBuf;

    #[test]
    fn test_biquad_lpf_matches_oracle() {
        let sample_rate = 44100.0;
        let cutoff_freq = 1000.0;
        let q = 1.0;

        let mut vector_path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        vector_path.push("tests");
        vector_path.push("vectors");
        vector_path.push("biquad_filters");
        vector_path.push("lpf");
        vector_path.push("lpf_64.npy");

        let expected = npy::load_f64(&vector_path).expect("failed to load biquad reference vector");
        let l = expected.len();

        let mut filter = Biquad::<f64>::lowpass(sample_rate, cutoff_freq, q);

        let mut actual = Vec::with_capacity(l);
        for n in 0..l {
            let input = if n == 0 { 1.0 } else { 0.0 };
            // Called on the filter instance instead of as a freestanding function
            actual.push(filter.biquad_filter(input));
        }

        assert_close!(actual, expected, rtol = 1e-12, atol = 1e-15);
    }
}
