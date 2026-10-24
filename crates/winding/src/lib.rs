//! PHYS-3 (docs/PHYS-POC.md): the winding number of a closed ring of phases,
//! W = round(Σ_i wrap(θ_{i+1} − θ_i) / 2π), with wrap into (−π, π]. An integer, unchanged by any
//! smooth deformation; it changes only when one neighbour step crosses π (a phase slip). Mechanism only.

use std::f64::consts::PI;

pub fn winding(theta: &[f64]) -> i64 {
    let n = theta.len();
    let mut total = 0.0;
    for i in 0..n {
        let mut d = theta[(i + 1) % n] - theta[i];
        d = (d + PI).rem_euclid(2.0 * PI) - PI;
        total += d;
    }
    (total / (2.0 * PI)).round() as i64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_one_and_minus_two() {
        let n = 32;
        let ring = |w: f64| -> Vec<f64> { (0..n).map(|i| 2.0 * PI * w * i as f64 / n as f64 + 0.3).collect() };
        assert_eq!(winding(&ring(0.0)), 0);
        assert_eq!(winding(&ring(1.0)), 1);
        assert_eq!(winding(&ring(-2.0)), -2);
    }
}
