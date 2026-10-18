//! Chaos: the fully chaotic logistic map x → 4x(1−x), and the tent map. Mechanism only; whether its values are frozen
//! (static disorder) or iterated each step (changing chaos) is the orchestrator's choice.

/// One step of the logistic map at r = 4.
pub fn logistic(x: f64) -> f64 {
    4.0 * x * (1.0 - x)
}

/// One step of the tent map with the given slope (≤ 2; a slope of exactly 2 collapses to 0 in
/// floating point).
pub fn tent(y: f64, slope: f64) -> f64 {
    slope * y.min(1.0 - y)
}

/// A chaotic value in (0, 1) for a key (QB-5): a hashed start, then 16 logistic steps. The logistic
/// map's invariant mean is 1/2.
pub fn keyed_unit(key: u64) -> f64 {
    let mut z = key.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^= z >> 31;
    let x0 = 0.01 + 0.98 * ((z >> 11) as f64 / (1u64 << 53) as f64);
    iterate(x0, 16).clamp(1e-9, 1.0 - 1e-9)
}

/// `x` after `n` steps.
pub fn iterate(mut x: f64, n: usize) -> f64 {
    for _ in 0..n {
        x = logistic(x);
    }
    x
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stays_inside_the_unit_interval() {
        let mut x = 0.123_456;
        for _ in 0..10_000 {
            x = logistic(x);
            assert!((0.0..=1.0).contains(&x));
        }
        assert!((iterate(0.3, 2) - logistic(logistic(0.3))).abs() < 1e-15);
    }

    #[test]
    fn keyed_units_have_the_logistic_mean() {
        let n = 100_000u64;
        let m: f64 = (0..n).map(keyed_unit).sum::<f64>() / n as f64;
        assert!((m - 0.5).abs() < 0.01, "{m}");
    }

    #[test]
    fn the_tent_map_at_slope_199_does_not_die() {
        let mut y = 0.37;
        let mut low = 0;
        for _ in 0..10_000 {
            y = tent(y, 1.99);
            assert!((0.0..1.0).contains(&y));
            low += usize::from(y < 1e-6);
        }
        assert!(low < 10, "{low}");
    }
}
