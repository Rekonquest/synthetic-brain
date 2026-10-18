//! Chaos: the fully chaotic logistic map x → 4x(1−x). Mechanism only; whether its values are frozen
//! (static disorder) or iterated each step (changing chaos) is the orchestrator's choice.

/// One step of the logistic map at r = 4.
pub fn logistic(x: f64) -> f64 {
    4.0 * x * (1.0 - x)
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
}
