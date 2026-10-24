//! ENT-1 (docs/ENT-1-PRECOMMIT.md): a local unitary. Qubit k of a register (amplitude index bit k)
//! is rotated by θ in the real plane: |0⟩ → cos θ|0⟩ + sin θ|1⟩, |1⟩ → −sin θ|0⟩ + cos θ|1⟩, on
//! every pair of amplitudes that differ only in bit k. The other qubits are untouched. Mechanism only.

use qb_amplitude::State;

pub fn rotate(s: &mut State, k: usize, theta: f64) {
    let (c, sn) = (theta.cos(), theta.sin());
    let bit = 1usize << k;
    for i in 0..s.a.len() {
        if i & bit == 0 {
            let (x, y) = (s.a[i], s.a[i | bit]);
            s.a[i] = x.scale(c).add(y.scale(-sn));
            s.a[i | bit] = x.scale(sn).add(y.scale(c));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use qb_amplitude::C;

    #[test]
    fn quarter_turn_and_norm() {
        let mut s = State { a: vec![C::new(1.0, 0.0), C::new(0.0, 0.0), C::new(0.0, 0.0), C::new(0.0, 0.0)] };
        rotate(&mut s, 1, std::f64::consts::FRAC_PI_2); // qubit 1 of |00⟩ → |10⟩ (index 2)
        assert!(s.a[0].abs() < 1e-12 && (s.a[2].re - 1.0).abs() < 1e-12);
        rotate(&mut s, 0, 0.3);
        let n: f64 = s.a.iter().map(|c| c.norm2()).sum();
        assert!((n - 1.0).abs() < 1e-12);
    }
}
