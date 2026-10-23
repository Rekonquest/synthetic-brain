//! FOLD-5's diagnosis (docs/FOLD-5-PRECOMMIT.md): the phase-locked interference reading,
//! Re⟨c, a⟩ / (‖c‖·‖a‖) with ⟨c, a⟩ = Σ conj(c_i)·a_i. A memory written with the key's own phase
//! overlaps it on the real axis; crosstalk from other memories lands at random phases, and only its
//! real component counts here (the magnitude reading counts both). In [−1, 1]; 0 if either side is
//! empty. Mechanism only.

use qb_amplitude::C;

pub fn overlap_phase(cue: &[C], state: &[C]) -> f64 {
    let (mut re, mut nc, mut na) = (0.0, 0.0, 0.0);
    for (c, a) in cue.iter().zip(state) {
        re += c.re * a.re + c.im * a.im;
        nc += c.norm2();
        na += a.norm2();
    }
    if nc <= 0.0 || na <= 0.0 {
        return 0.0;
    }
    re / (nc * na).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn in_phase_quadrature_and_opposite() {
        let a = [C::new(1.0, 0.0), C::new(0.0, 1.0)];
        assert!((overlap_phase(&a, &a) - 1.0).abs() < 1e-12);
        let quadrature = [C::new(0.0, 1.0), C::new(-1.0, 0.0)]; // a times i: the magnitude reading gives 1
        assert!(overlap_phase(&a, &quadrature).abs() < 1e-12);
        let opposite = [C::new(-1.0, 0.0), C::new(0.0, -1.0)];
        assert!((overlap_phase(&a, &opposite) + 1.0).abs() < 1e-12);
    }
}
