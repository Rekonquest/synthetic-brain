//! FOLD-5 (docs/FOLD-5-PRECOMMIT.md): the centred correlation of two real profiles,
//! Σ(a − ā)(b − b̄) / √(Σ(a − ā)²·Σ(b − b̄)²), in [−1, 1]; 0 if either is flat. Centring makes
//! non-negative profiles zero-mean, so folded memories cancel instead of piling up. Mechanism only.

pub fn correlate(a: &[f64], b: &[f64]) -> f64 {
    let n = a.len().min(b.len()) as f64;
    if n == 0.0 {
        return 0.0;
    }
    let (ma, mb) = (a.iter().sum::<f64>() / n, b.iter().sum::<f64>() / n);
    let (mut ab, mut aa, mut bb) = (0.0, 0.0, 0.0);
    for (x, y) in a.iter().zip(b) {
        let (dx, dy) = (x - ma, y - mb);
        ab += dx * dy;
        aa += dx * dx;
        bb += dy * dy;
    }
    if aa <= 0.0 || bb <= 0.0 {
        return 0.0;
    }
    ab / (aa * bb).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_shape_opposite_and_flat() {
        assert!((correlate(&[1.0, 2.0, 3.0], &[10.0, 20.0, 30.0]) - 1.0).abs() < 1e-12);
        assert!((correlate(&[1.0, 2.0, 3.0], &[3.0, 2.0, 1.0]) + 1.0).abs() < 1e-12);
        assert_eq!(correlate(&[1.0, 2.0, 3.0], &[5.0, 5.0, 5.0]), 0.0);
    }
}
