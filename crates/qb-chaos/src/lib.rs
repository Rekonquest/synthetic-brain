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
