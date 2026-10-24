//! PHYS-2 (docs/PHYS-POC.md): the gauge-invariant reading. A local phase e^{iφ_i} on every element
//! cancels from u_i·conj(u_{i+1}) except for φ_i − φ_{i+1}, so the length-1 Wilson line
//! Re Σ_i u_i·conj(u_{i+1}) / Σ_i |u_i||u_{i+1}| survives any smooth local phase. In [−1, 1]; 0 if
//! empty. Mechanism only.

use qb_amplitude::C;

pub fn wilson(u: &[C]) -> f64 {
    let (mut re, mut norm) = (0.0, 0.0);
    for w in u.windows(2) {
        re += w[0].re * w[1].re + w[0].im * w[1].im;
        norm += w[0].abs() * w[1].abs();
    }
    if norm <= 0.0 { 0.0 } else { re / norm }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_smooth_gauge_cancels() {
        let flat: Vec<C> = (0..100).map(|_| C::new(1.0, 0.0)).collect();
        let gauged: Vec<C> = (0..100).map(|i| C::polar(1.0, 0.01 * (i * i) as f64 / 10.0)).collect();
        assert!((wilson(&flat) - 1.0).abs() < 1e-12);
        assert!(wilson(&gauged) > 0.99);
        let alternating: Vec<C> = (0..100).map(|i| C::new(if i % 2 == 0 { 1.0 } else { -1.0 }, 0.0)).collect();
        assert!((wilson(&alternating) + 1.0).abs() < 1e-12);
    }
}
