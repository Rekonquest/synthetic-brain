//! FOLD-5 (docs/FOLD-5-PRECOMMIT.md): the interference reading over complex amplitudes,
//! |⟨c, a⟩| / (‖c‖·‖a‖) with ⟨c, a⟩ = Σ conj(c_i)·a_i. Amplitudes in phase add; random phases cancel,
//! so a cue reads its own memory out of a superposition and the others average away. In [0, 1];
//! 0 if either side is empty. Mechanism only.

use qb_amplitude::C;

pub fn overlap(cue: &[C], state: &[C]) -> f64 {
    let (mut re, mut im, mut nc, mut na) = (0.0, 0.0, 0.0, 0.0);
    for (c, a) in cue.iter().zip(state) {
        re += c.re * a.re + c.im * a.im;
        im += c.re * a.im - c.im * a.re;
        nc += c.norm2();
        na += a.norm2();
    }
    if nc <= 0.0 || na <= 0.0 {
        return 0.0;
    }
    (re * re + im * im).sqrt() / (nc * na).sqrt()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn same_global_phase_orthogonal_and_half() {
        let a = [C::new(1.0, 0.0), C::new(0.0, 1.0)];
        let rotated = [C::new(0.0, 1.0), C::new(-1.0, 0.0)]; // a times i
        assert!((overlap(&a, &rotated) - 1.0).abs() < 1e-12);
        let b = [C::new(1.0, 0.0), C::new(0.0, -1.0)];
        assert!(overlap(&a, &b) < 1e-12);
        let sum = [C::new(2.0, 0.0), C::new(0.0, 0.0)]; // a + b
        assert!((overlap(&a, &sum) - 0.5f64.sqrt()).abs() < 1e-12);
        assert_eq!(overlap(&a, &[C::new(0.0, 0.0); 2]), 0.0);
    }
}
