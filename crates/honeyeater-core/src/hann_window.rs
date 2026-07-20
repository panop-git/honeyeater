//Check for symmetric hanning window
pub fn hann_window(n: usize, l: usize) -> f64 {
    hann(n, l, true)
}

//Check for non-periodic hanning window
pub fn hann_window_periodic(n: usize, l: usize) -> f64 {
    hann(n, l, false)
}

pub fn hann(n: usize, l: usize, symmetric: bool) -> f64 {
    debug_assert!(n < l, "sample index n ({n}) must be less than window length l ({l})");
    match l {
        0 => panic!("Hann window length must be greater than zero"),
        1 => 1.0,
        _ => {
            let denom = if symmetric {l - 1} else { l };
            let angle = 2.0 * std::f64::consts::PI * n as f64 / denom as f64;
            0.5 - 0.5 * angle.cos()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn calls_hann_window() {
        for n in 0..5 {
            let value = hann_window(n, 5);
            println!("hann_window({n}, 5) = {value}");
        }
    }
}