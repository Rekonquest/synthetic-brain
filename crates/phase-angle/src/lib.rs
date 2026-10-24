//! GEO-2 (docs/GEO-2-PRECOMMIT.md): the grid-cell phase code. Each module k reports a phase φ_k ≡ kθ
//! (mod 2π) with a weight w_k; alone it leaves k branches (φ_k + 2πn)/k. Together the modules fix θ:
//! every branch of every module is a candidate, and the read angle is the candidate with the least
//! Σ_k w_k·(1 − cos(kθ − φ_k)). Mechanism only: which modules, and their phases and weights, are the
//! caller's.

use std::f64::consts::PI;

pub fn read(modules: &[(usize, f64, f64)]) -> f64 {
    let cost = |t: f64| -> f64 { modules.iter().map(|&(k, phi, w)| w * (1.0 - (k as f64 * t - phi).cos())).sum() };
    let mut best = (f64::MAX, 0.0);
    for &(k, phi, _) in modules {
        for n in 0..k {
            let t = (phi + 2.0 * PI * n as f64) / k as f64;
            let c = cost(t);
            if c < best.0 {
                best = (c, t);
            }
        }
    }
    best.1.rem_euclid(2.0 * PI)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_modules_fix_what_one_cannot() {
        let theta: f64 = 4.0; // beyond pi: module 2 alone cannot tell 4.0 from 4.0 - pi
        let m = |k: usize| (k, (k as f64 * theta).rem_euclid(2.0 * PI), 1.0);
        assert!((read(&[m(2), m(3)]) - theta).abs() < 1e-9);
        let one = read(&[m(2)]);
        assert!((one - theta).abs() < 1e-9 || (one - (theta - PI)).abs() < 1e-9);
    }
}
