//! ENT-1 (docs/ENT-1-PRECOMMIT.md): Born measurement of one qubit. P(qubit k = 0) is the weight of
//! the amplitudes with bit k clear; the outcome is 0 if the given uniform u is below it. The register
//! collapses onto the outcome (the other amplitudes become 0) and is renormalized, so the other
//! qubits are left as the collapse leaves them. Mechanism only: u is the caller's.

use qb_amplitude::{State, C};

pub fn measure(s: &mut State, k: usize, u: f64) -> usize {
    let bit = 1usize << k;
    let total: f64 = s.a.iter().map(|c| c.norm2()).sum();
    let p0: f64 = s.a.iter().enumerate().filter(|(i, _)| i & bit == 0).map(|(_, c)| c.norm2()).sum::<f64>() / total;
    let out = if u < p0 { 0 } else { 1 };
    for (i, c) in s.a.iter_mut().enumerate() {
        if ((i & bit != 0) as usize) != out {
            *c = C::new(0.0, 0.0);
        }
    }
    s.normalize();
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bell_pair_collapses_together() {
        let h = 0.5f64.sqrt();
        let bell = State { a: vec![C::new(h, 0.0), C::new(0.0, 0.0), C::new(0.0, 0.0), C::new(h, 0.0)] };
        let mut s = bell.clone();
        assert_eq!(measure(&mut s, 0, 0.9), 1);
        assert_eq!(measure(&mut s, 1, 0.0), 1); // the partner is now certain
        let mut s = bell;
        assert_eq!(measure(&mut s, 0, 0.1), 0);
        assert_eq!(measure(&mut s, 1, 0.999), 0);
    }
}
