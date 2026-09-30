use num_complex::Complex;
use phastft::{fft_f32_dit_interleaved, fft_f64_dit_interleaved, planner::Direction};
use std::marker::PhantomData;

use super::FftWrapper;

// Bridge trait to map generics to phastft's hardcoded function names
pub(crate) trait PhastFftFloat {
    fn execute(buffer: &mut [Complex<Self>], direction: Direction)
    where
        Self: Sized;
}

impl PhastFftFloat for f32 {
    fn execute(buffer: &mut [Complex<f32>], direction: Direction) {
        fft_f32_dit_interleaved(buffer, direction);
    }
}

impl PhastFftFloat for f64 {
    fn execute(buffer: &mut [Complex<f64>], direction: Direction) {
        fft_f64_dit_interleaved(buffer, direction);
    }
}

pub(crate) struct PhastFftBackend<T> {
    size: usize,
    _marker: PhantomData<T>,
}

impl<T: PhastFftFloat> PhastFftBackend<T> {
    pub(crate) fn new(size: usize) -> Self {
        assert!(
            size.is_power_of_two(),
            "phastft only supports power-of-2 sizes"
        );
        Self {
            size,
            _marker: PhantomData,
        }
    }
}

impl<T: PhastFftFloat> FftWrapper<T> for PhastFftBackend<T> {
    fn fft(&self, buffer: &mut [Complex<T>]) {
        T::execute(buffer, Direction::Forward);
    }

    fn ifft(&self, buffer: &mut [Complex<T>]) {
        T::execute(buffer, Direction::Inverse);
    }

    fn size(&self) -> usize {
        self.size
    }
}
