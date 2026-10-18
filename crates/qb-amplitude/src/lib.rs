//! Quantum-derived base mechanics, as ordinary mathematics on a classical PC (vault Rule 4: no
//! quantum computation is claimed). Mechanism only; nothing here decides anything.
//!
//! - a state is one complex amplitude per option;
//! - `born_weights`: probabilities |a_i|² (the Born rule);
//! - `project`: destructive measurement — the state becomes the measured option alone;
//! - `ring_mix`: a partial unitary between ring neighbours (even pairs, then odd pairs);
//! - `rotate`: a phase rotation of one amplitude;
//! - `scale` and `power_magnitudes`: the entry points classical hooks act through;
//! - `excite`: add an amplitude to one option (a thermal occupation);
//! - `blend`: inertia, mixing in the previous state.

/// A complex number.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct C {
    pub re: f64,
    pub im: f64,
}

impl C {
    pub fn new(re: f64, im: f64) -> C {
        C { re, im }
    }
    pub fn mul(self, o: C) -> C {
        C::new(self.re * o.re - self.im * o.im, self.re * o.im + self.im * o.re)
    }
    pub fn add(self, o: C) -> C {
        C::new(self.re + o.re, self.im + o.im)
    }
    pub fn scale(self, k: f64) -> C {
        C::new(self.re * k, self.im * k)
    }
    pub fn norm2(self) -> f64 {
        self.re * self.re + self.im * self.im
    }
    pub fn abs(self) -> f64 {
        self.norm2().sqrt()
    }
    pub fn polar(r: f64, theta: f64) -> C {
        C::new(r * theta.cos(), r * theta.sin())
    }
}

/// One amplitude per option.
#[derive(Debug, Clone, PartialEq)]
pub struct State {
    pub a: Vec<C>,
}

impl State {
    pub fn uniform(n: usize) -> State {
        State { a: vec![C::new(1.0 / (n as f64).sqrt(), 0.0); n] }
    }

    /// Rescale so Σ|a|² = 1.
    pub fn normalize(&mut self) {
        let n = self.a.iter().map(|c| c.norm2()).sum::<f64>().sqrt();
        if n > 0.0 {
            for c in &mut self.a {
                *c = c.scale(1.0 / n);
            }
        }
    }

    /// The Born weights |a_i|² / Σ|a|².
    pub fn born_weights(&self) -> Vec<f64> {
        let t: f64 = self.a.iter().map(|c| c.norm2()).sum();
        self.a.iter().map(|c| c.norm2() / t).collect()
    }

    /// Destructive measurement of option k: every other amplitude becomes 0; k keeps its phase.
    pub fn project(&mut self, k: usize) {
        let keep = self.a[k];
        let r = keep.abs();
        for c in &mut self.a {
            *c = C::new(0.0, 0.0);
        }
        self.a[k] = if r > 0.0 { keep.scale(1.0 / r) } else { C::new(1.0, 0.0) };
    }

    /// The partial unitary [[cos φ, i sin φ], [i sin φ, cos φ]] on ring pairs (0,1), (2,3), …, then
    /// (1,2), (3,4), …, (n−1, 0). Written for an even n.
    pub fn ring_mix(&mut self, phi: f64) {
        let n = self.a.len();
        if n < 2 {
            return;
        }
        let (c, s) = (C::new(phi.cos(), 0.0), C::new(0.0, phi.sin()));
        for start in [0usize, 1] {
            let mut i = start;
            while i < n {
                let j = (i + 1) % n;
                let (x, y) = (self.a[i], self.a[j]);
                self.a[i] = c.mul(x).add(s.mul(y));
                self.a[j] = s.mul(x).add(c.mul(y));
                i += 2;
            }
        }
    }

    /// Multiply amplitude k by e^{iθ}.
    pub fn rotate(&mut self, k: usize, theta: f64) {
        self.a[k] = self.a[k].mul(C::polar(1.0, theta));
    }

    /// Multiply amplitude k by a real factor.
    pub fn scale(&mut self, k: usize, f: f64) {
        self.a[k] = self.a[k].scale(f);
    }

    /// Add an amplitude to option k (a thermal occupation of the mode; QB-2).
    pub fn excite(&mut self, k: usize, c: C) {
        self.a[k] = self.a[k].add(c);
    }

    /// Inertia (QB-3): a ← (1 − μ)·a + μ·prev, a complex combination; the caller renormalizes.
    pub fn blend(&mut self, prev: &State, mu: f64) {
        for (c, p) in self.a.iter_mut().zip(&prev.a) {
            *c = c.scale(1.0 - mu).add(p.scale(mu));
        }
    }

    /// |a_i| ← |a_i|^p, phases kept.
    pub fn power_magnitudes(&mut self, p: f64) {
        for c in &mut self.a {
            let r = c.abs();
            if r > 0.0 {
                *c = c.scale(r.powf(p) / r);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_ring_mixer_is_unitary_and_touches_every_pair() {
        let mut s = State { a: (0..8).map(|i| C::new(i as f64 * 0.3 - 1.0, 0.1 * i as f64)).collect() };
        s.normalize();
        let before: f64 = s.a.iter().map(|c| c.norm2()).sum();
        for _ in 0..100 {
            s.ring_mix(std::f64::consts::PI / 12.0);
        }
        let after: f64 = s.a.iter().map(|c| c.norm2()).sum();
        assert!((before - after).abs() < 1e-12);
        // From a basis state, one mix reaches both ring neighbours (including the wrap-around).
        let mut e = State { a: vec![C::new(0.0, 0.0); 8] };
        e.a[0] = C::new(1.0, 0.0);
        e.ring_mix(0.3);
        assert!(e.a[1].norm2() > 0.0 && e.a[7].norm2() > 0.0);
    }

    #[test]
    fn projection_is_idempotent_and_keeps_phase() {
        let mut s = State::uniform(8);
        s.rotate(3, 1.0);
        s.project(3);
        let once = s.clone();
        s.project(3);
        // Re-projecting divides by a norm of 1 up to rounding, so compare to 1e-12.
        assert!(s.a.iter().zip(&once.a).all(|(x, y)| (x.re - y.re).abs() < 1e-12 && (x.im - y.im).abs() < 1e-12));
        assert!((s.a[3].abs() - 1.0).abs() < 1e-12 && (s.a[3].im.atan2(s.a[3].re) - 1.0).abs() < 1e-12);
        assert!(s.a.iter().enumerate().all(|(i, c)| i == 3 || c.norm2() == 0.0));
    }

    #[test]
    fn a_phase_kick_keeps_magnitudes_and_excite_lifts_a_zero() {
        let mut s = State::uniform(8);
        let before: Vec<f64> = s.a.iter().map(|c| c.abs()).collect();
        for j in 0..8 {
            s.rotate(j, 0.7 * j as f64);
        }
        assert!(s.a.iter().zip(&before).all(|(c, b)| (c.abs() - b).abs() < 1e-15));
        s.project(2);
        s.excite(5, C::polar(0.01, 1.0));
        assert!(s.a[5].abs() > 0.0);
    }

    #[test]
    fn blend_with_zero_inertia_is_the_identity() {
        let mut s = State::uniform(4);
        s.rotate(1, 0.4);
        let keep = s.clone();
        s.blend(&State::uniform(4), 0.0);
        assert_eq!(s, keep);
    }

    #[test]
    fn born_weights_sum_to_one() {
        let mut s = State::uniform(8);
        s.scale(2, 3.0);
        assert!((s.born_weights().iter().sum::<f64>() - 1.0).abs() < 1e-12);
    }
}
