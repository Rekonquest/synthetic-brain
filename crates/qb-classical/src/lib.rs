//! Classical first-principle hooks (Jesse's cognitive roots), on plain non-negative weights.
//! Mechanism only.
//!
//! - `reinforce` (REINFORCE): multiply one weight by (1 + η);
//! - `decay` (DECAY): w ← w^(1−λ), which lifts small weights toward the rest (forgetting);
//! - `conserve` (CONSERVE): rescale so Σw = 1;
//! - `draw`: pick an index with probability w_i / Σw, given a uniform u in [0, 1);
//! - `argmax`: the first largest weight;
//! - `temper`: the Boltzmann-style reweighting p_i ∝ w_i^(1/T), normalized (QB-2's decision temperature);
//! - `diffuse`: classical ring diffusion (Fick), the replacement for the unitary mixer (QB-3);
//! - `gate`: the persistence gate — an option counts when it appears at least k times in a window (QB-3).

pub fn reinforce(w: &mut [f64], k: usize, eta: f64) {
    w[k] *= 1.0 + eta;
}

pub fn decay(w: &mut [f64], lambda: f64) {
    for x in w.iter_mut() {
        if *x > 0.0 {
            *x = x.powf(1.0 - lambda);
        }
    }
}

pub fn conserve(w: &mut [f64]) {
    let t: f64 = w.iter().sum();
    if t > 0.0 {
        for x in w.iter_mut() {
            *x /= t;
        }
    }
}

pub fn draw(w: &[f64], u: f64) -> usize {
    let t: f64 = w.iter().sum();
    let mut acc = 0.0;
    for (i, x) in w.iter().enumerate() {
        acc += x / t;
        if u < acc {
            return i;
        }
    }
    w.len() - 1
}

pub fn argmax(w: &[f64]) -> usize {
    let mut best = 0;
    for (i, x) in w.iter().enumerate() {
        if *x > w[best] {
            best = i;
        }
    }
    best
}

pub fn temper(w: &[f64], t: f64) -> Vec<f64> {
    let mut p: Vec<f64> = w.iter().map(|x| if *x > 0.0 { x.powf(1.0 / t) } else { 0.0 }).collect();
    conserve(&mut p);
    p
}

pub fn diffuse(w: &mut [f64], s: f64) {
    let n = w.len();
    let old = w.to_vec();
    for i in 0..n {
        w[i] = (1.0 - 2.0 * s) * old[i] + s * (old[(i + n - 1) % n] + old[(i + 1) % n]);
    }
}

pub fn gate(window: &[usize], option: usize, k: usize) -> bool {
    window.iter().filter(|&&x| x == option).count() >= k
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hooks_behave() {
        let mut w = vec![0.25; 4];
        reinforce(&mut w, 1, 1.0);
        conserve(&mut w);
        assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-12 && argmax(&w) == 1);
        let before = w[1] / w[0];
        decay(&mut w, 0.5);
        assert!(w[1] / w[0] < before);
        assert_eq!(draw(&[0.0, 1.0, 0.0], 0.5), 1);
        assert_eq!(argmax(&[1.0, 2.0, 2.0]), 1);
    }

    #[test]
    fn gate_and_diffuse() {
        assert!(gate(&[3, 1, 3], 3, 2) && !gate(&[3, 1, 2], 3, 2));
        let mut w = vec![0.0, 1.0, 0.0, 0.0];
        diffuse(&mut w, 0.1);
        assert!((w.iter().sum::<f64>() - 1.0).abs() < 1e-15 && (w[0] - 0.1).abs() < 1e-15 && (w[2] - 0.1).abs() < 1e-15);
    }

    #[test]
    fn temper_at_one_is_the_identity_and_at_two_takes_the_root() {
        let w = [0.1, 0.2, 0.3, 0.4];
        assert!(temper(&w, 1.0).iter().zip(&w).all(|(a, b)| (a - b).abs() < 1e-15));
        let r: f64 = w.iter().map(|x| x.sqrt()).sum();
        assert!(temper(&w, 2.0).iter().zip(&w).all(|(a, b)| (a - b.sqrt() / r).abs() < 1e-15));
    }
}
