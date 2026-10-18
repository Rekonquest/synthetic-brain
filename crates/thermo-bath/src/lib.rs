//! Thermal base, rung T2 (docs/T2-BATH.md): a temperature for the field on T1's lattice.
//!
//! Law: fluctuation–dissipation, paired on *every* dissipative channel, so the medium equilibrates at
//! the temperature it is given:
//! 1. damping at each site — the exact Ornstein–Uhlenbeck step toward the local potential
//!    μ_i = φ_i − h_i: φ ← h + (φ − h)e^{−γ} + √(T(1 − e^{−2γ}))·ξ;
//! 2. exchange with conserved noise on each edge — f = (κ/2)(μ_j − μ_i) + σ_e·ξ_e, φ_i += f,
//!    φ_j −= f, with σ_e² = Tκ(2 − κ)/2. Edges of an axis are swept in two disjoint halves (even, then
//!    odd coordinates), so each update sees current values. For independent φ_i, φ_j ~ N(h, T) the
//!    update leaves each variance at T and their covariance at 0: with δ = φ_j − φ_i,
//!    var(φ_i') = (1 − κ/2)²T + (κ/2)²T + σ_e² = T and cov = κ(1 − κ/2)T − σ_e² = 0.
//!
//! The axis count never enters the law. Mechanism only: T, γ, κ, h and the noise are the caller's.

use thermo_lattice::Lattice;

/// A standard normal from two uniforms in (0, 1) (Box–Muller).
pub fn normal(u1: f64, u2: f64) -> f64 {
    (-2.0 * u1.max(1e-300).ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
}

/// One damping step at every site. `gamma(site)`, `noise(site)` (a standard normal), field `h`.
pub fn damp(field: &mut Lattice, temperature: f64, gamma: &dyn Fn(usize) -> f64, h: &dyn Fn(usize) -> f64, noise: &dyn Fn(usize) -> f64) {
    for site in 0..field.e.len() {
        let g = gamma(site);
        let a = (-g).exp();
        let hs = h(site);
        field.e[site] = hs + (field.e[site] - hs) * a + (temperature * (1.0 - a * a)).sqrt() * noise(site);
    }
}

/// One exchange step with conserved noise: per axis, even edges then odd edges.
/// `noise(site, axis)` is a standard normal for the edge (site, forward(site, axis)).
pub fn exchange(field: &mut Lattice, temperature: f64, kappa: f64, h: &dyn Fn(usize) -> f64, noise: &dyn Fn(usize, usize) -> f64) {
    let sigma = (temperature * kappa * (2.0 - kappa) / 2.0).sqrt();
    for axis in 0..field.dims.len() {
        if field.dims[axis] < 2 {
            continue;
        }
        for parity in [0, 1] {
            for site in 0..field.e.len() {
                if field.coord(site, axis) % 2 != parity {
                    continue;
                }
                let j = field.forward(site, axis);
                let f = kappa / 2.0 * ((field.e[j] - h(j)) - (field.e[site] - h(site))) + sigma * noise(site, axis);
                field.e[site] += f;
                field.e[j] -= f;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cold_exchange_conserves_and_normal_is_standard() {
        let mut l = Lattice::new(&[8, 8]);
        for (i, x) in l.e.iter_mut().enumerate() {
            *x = (i as f64 * 0.37).sin();
        }
        let t0 = l.total();
        for _ in 0..100 {
            exchange(&mut l, 0.0, 0.3, &|_| 0.0, &|_, _| 0.0);
        }
        assert!((l.total() - t0).abs() < 1e-9);
        // Box–Muller on a fine grid of uniforms has mean ~0 and variance ~1.
        let n = 400;
        let mut s = (0.0, 0.0);
        for a in 0..n {
            for b in 0..n {
                let z = normal((a as f64 + 0.5) / n as f64, (b as f64 + 0.5) / n as f64);
                s.0 += z;
                s.1 += z * z;
            }
        }
        let k = (n * n) as f64;
        assert!((s.0 / k).abs() < 1e-3 && (s.1 / k - 1.0).abs() < 0.01, "{} {}", s.0 / k, s.1 / k);
    }
}
