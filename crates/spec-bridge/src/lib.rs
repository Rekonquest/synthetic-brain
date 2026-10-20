//! Jesse's spectral bridge, from universum-substrate `src/field.rs` (`spectral_bridge`): the overlap
//! of two spectra, 1 − ½·Σ|a_i/Σa − b_i/Σb|, in [0, 1]; 0 if either is empty. Copied as its own
//! crate (cross-compatible atom), in f64.

pub fn bridge(a: &[f64], b: &[f64]) -> f64 {
    let (ta, tb): (f64, f64) = (a.iter().sum(), b.iter().sum());
    if ta <= 1e-6 || tb <= 1e-6 {
        return 0.0;
    }
    let span = a.len().max(b.len());
    let diff: f64 = (0..span).map(|i| (a.get(i).copied().unwrap_or(0.0) / ta - b.get(i).copied().unwrap_or(0.0) / tb).abs()).sum();
    (1.0 - 0.5 * diff).clamp(0.0, 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identical_disjoint_and_half() {
        assert_eq!(bridge(&[1.0, 2.0], &[2.0, 4.0]), 1.0);
        assert_eq!(bridge(&[1.0, 0.0], &[0.0, 1.0]), 0.0);
        assert!((bridge(&[1.0, 1.0], &[1.0, 0.0]) - 0.5).abs() < 1e-12);
    }
}
