//! PROB-1 (docs/PROB-1-PRECOMMIT.md): the signal behind a set of readings, by the method of moments.
//! Each item has m readings: one carries a mean μ, the rest are noise of variance σ². Then
//! E[Σ_v z_v²] = μ² + m·σ² (up to the signal item's own noise), so μ = √max(mean_sum_sq − m·σ², 0).
//! Mechanism only: the readings and σ² are the caller's.

pub fn signal(mean_sum_sq: f64, noise_var: f64, m: usize) -> f64 {
    (mean_sum_sq - m as f64 * noise_var).max(0.0).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recovers_the_mean() {
        // m 4, sigma^2 0.01, mu 0.3: E[sum z^2] = 0.09 + 4 * 0.01 = 0.13.
        assert!((signal(0.13, 0.01, 4) - 0.3).abs() < 1e-12);
        assert_eq!(signal(0.03, 0.01, 4), 0.0);
    }
}
