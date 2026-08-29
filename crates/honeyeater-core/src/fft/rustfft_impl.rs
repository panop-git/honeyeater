use num_complex::Complex;
use rustfft::{Fft, FftNum, FftPlanner};
use std::sync::Arc;

use super::FftWrapper;

pub(crate) struct RustFftBackend<T> {
    forward: Arc<dyn Fft<T>>,
    inverse: Arc<dyn Fft<T>>,
    size: usize,
}

impl<T: FftNum> RustFftBackend<T> {
    pub(crate) fn new(size: usize) -> Self {
        let mut planner = FftPlanner::<T>::new();
        let forward = planner.plan_fft_forward(size);
        let inverse = planner.plan_fft_inverse(size);

        Self {
            forward,
            inverse,
            size,
        }
    }
}

impl<T: FftNum> FftWrapper<T> for RustFftBackend<T> {
    fn fft(&self, buffer: &mut [Complex<T>]) {
        self.forward.process(buffer);
    }

    fn ifft(&self, buffer: &mut [Complex<T>]) {
        self.inverse.process(buffer);

        // Cast the normalisation factor into the generic type T
        let n_inv = T::from_f64(1.0 / buffer.len() as f64).unwrap();
        for x in buffer.iter_mut() {
            *x = *x * n_inv;
        }
    }

    fn size(&self) -> usize {
        self.size
    }
}
