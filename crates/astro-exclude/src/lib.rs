//! ASTRO-1 arm A (docs/ASTRO-POC.md): one Pauli-exclusion step on a superposed store. Each item i
//! occupies the state b_i; its crosstalk is r_i = <b_i, s>/|b_i|^2 - target_i (complex: the overlap
//! of everything but its own target component). All r_i are read from the same store, then
//! s <- s - eta * sum_i r_i * b_i. Iterated, it approaches zero-forcing. Mechanism only: eta, the
//! number of steps and the targets are the caller's.

use qb_amplitude::C;

pub fn exclude(s: &mut [C], bound: &[Vec<C>], target: &[f64], eta: f64) {
    let r: Vec<C> = bound
        .iter()
        .zip(target)
        .map(|(b, t)| {
            let (mut re, mut im, mut nn) = (0.0, 0.0, 0.0);
            for (x, y) in b.iter().zip(s.iter()) {
                // conj(x) * y
                re += x.re * y.re + x.im * y.im;
                im += x.re * y.im - x.im * y.re;
                nn += x.norm2();
            }
            if nn > 0.0 { C::new(re / nn - t, im / nn) } else { C::new(0.0, 0.0) }
        })
        .collect();
    for (b, ri) in bound.iter().zip(&r) {
        for (x, y) in s.iter_mut().zip(b) {
            let p = ri.mul(*y);
            *x = C::new(x.re - eta * p.re, x.im - eta * p.im);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_the_shared_component_and_keeps_each_own() {
        // Two non-orthogonal unit states; s = b0 + b1 has crosstalk on each.
        let b0 = vec![C::new(1.0, 0.0), C::new(0.0, 0.0)];
        let b1 = vec![C::new(0.6, 0.0), C::new(0.8, 0.0)];
        let mut s: Vec<C> = b0.iter().zip(&b1).map(|(a, b)| a.add(*b)).collect();
        for _ in 0..200 {
            exclude(&mut s, &[b0.clone(), b1.clone()], &[1.0, 1.0], 0.5);
        }
        let ov = |b: &[C]| b.iter().zip(&s).map(|(x, y)| x.re * y.re + x.im * y.im).sum::<f64>();
        assert!((ov(&b0) - 1.0).abs() < 1e-9 && (ov(&b1) - 1.0).abs() < 1e-9);
        // eta 0 leaves the store untouched.
        let mut t = s.clone();
        exclude(&mut t, &[b0, b1], &[0.0, 0.0], 0.0);
        assert_eq!(t, s);
    }
}
