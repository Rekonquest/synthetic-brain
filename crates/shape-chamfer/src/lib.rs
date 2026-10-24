//! GEO-1 (docs/GEO-1-PRECOMMIT.md): the symmetric Chamfer distance, the mean distance from each point
//! of one set to its nearest point in the other, averaged both ways. Mechanism only.

pub fn chamfer(a: &[(f64, f64)], b: &[(f64, f64)]) -> f64 {
    let one = |p: &[(f64, f64)], q: &[(f64, f64)]| -> f64 {
        p.iter().map(|x| q.iter().map(|y| ((x.0 - y.0).powi(2) + (x.1 - y.1).powi(2)).sqrt()).fold(f64::MAX, f64::min)).sum::<f64>() / p.len() as f64
    };
    0.5 * (one(a, b) + one(b, a))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_for_the_same_set_in_any_order() {
        let a = [(0.0, 0.0), (1.0, 0.0), (0.0, 2.0)];
        let b = [(0.0, 2.0), (0.0, 0.0), (1.0, 0.0)];
        assert_eq!(chamfer(&a, &b), 0.0);
        assert!((chamfer(&a, &[(0.0, 0.0), (1.0, 0.0), (0.0, 3.0)]) - 1.0 / 3.0).abs() < 1e-12);
    }
}
