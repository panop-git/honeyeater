//! Statistical helpers used by the assertion macros.
//!
//! Kept private to the test harness; not part of any published API.

/// Compute the one-sample Kolmogorov-Smirnov D-statistic and its critical
/// value at significance level `alpha`.
///
/// This is the kernel used by [`crate::assert_distribution_ks!`]. The
/// D-statistic is `sup_x |F_n(x) − F(x)|`, where `F_n` is the empirical CDF
/// of the samples and `F` is the target CDF. The critical value uses the
/// large-sample asymptotic approximation
/// `D_crit ≈ c(α) / √n`, where `c(α)` for the common α values are well-known
/// (e.g. `c(0.05) ≈ 1.358`, `c(0.01) ≈ 1.628`).
///
/// # Panics
///
/// Panics if `samples` is empty (D-statistic undefined) or if `alpha` is
/// outside the supported set `{0.10, 0.05, 0.01, 0.001}`, samples are not finite,
/// or the CDF returns a non-finite value or a value outside `[0, 1]`.
#[must_use]
pub fn ks_one_sample(samples: &[f64], cdf: fn(f64) -> f64, alpha: f64) -> (f64, f64) {
    assert!(
        !samples.is_empty(),
        "ks_one_sample: samples must be non-empty",
    );
    assert!(
        samples.iter().all(|x| x.is_finite()),
        "ks_one_sample: samples must be finite"
    );
    let n = samples.len();
    #[allow(clippy::cast_precision_loss)]
    let n_f = n as f64;

    let mut sorted: Vec<f64> = samples.to_vec();
    sorted.sort_by(|a, b| a.partial_cmp(b).expect("ks_one_sample: NaN in samples"));

    let mut d_stat = 0.0_f64;
    for (i, x) in sorted.iter().enumerate() {
        #[allow(clippy::cast_precision_loss)]
        let f_n_lower = i as f64 / n_f;
        #[allow(clippy::cast_precision_loss)]
        let f_n_upper = (i + 1) as f64 / n_f;
        let f_target = cdf(*x);
        assert!(
            f_target.is_finite() && (0.0..=1.0).contains(&f_target),
            "ks_one_sample: CDF must return a finite probability"
        );
        let diff_lower = (f_n_lower - f_target).abs();
        let diff_upper = (f_n_upper - f_target).abs();
        d_stat = d_stat.max(diff_lower).max(diff_upper);
    }

    // Asymptotic critical values for one-sample KS, large n.
    let c = match alpha {
        a if (a - 0.10).abs() < 1e-12 => 1.224_f64,
        a if (a - 0.05).abs() < 1e-12 => 1.358_f64,
        a if (a - 0.01).abs() < 1e-12 => 1.628_f64,
        a if (a - 0.001).abs() < 1e-12 => 1.949_f64,
        _ => panic!(
            "ks_one_sample: unsupported alpha = {alpha}; \
             supported values are 0.10, 0.05, 0.01, 0.001"
        ),
    };
    let d_critical = c / n_f.sqrt();

    (d_stat, d_critical)
}

#[cfg(test)]
mod tests {
    use super::ks_one_sample;

    #[test]
    #[should_panic(expected = "samples must be finite")]
    fn rejects_single_nan_sample() {
        ks_one_sample(&[f64::NAN], |x| x, 0.01);
    }

    #[test]
    #[should_panic(expected = "finite probability")]
    fn rejects_nan_cdf() {
        ks_one_sample(&[0.5], |_| f64::NAN, 0.01);
    }

    #[test]
    #[should_panic(expected = "finite probability")]
    fn rejects_invalid_cdf_probability() {
        ks_one_sample(&[0.5], |_| 2.0, 0.01);
    }
}
