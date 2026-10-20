//! FOLD-3 (docs/FOLD-3-PRECOMMIT.md): the containment reading, composed beside Jesse's spectral
//! bridge (which stays the whole-profile reading). It asks whether a cue's profile is *present* in
//! a region's spectrum, not whether the two profiles are equal:
//!   floor = the least-filled band of R (a band no stored memory uses: only the stir's flat
//!   spectrum); R′ = max(R − floor, 0);
//!   containment(c, R) = Σ_b ĉ_b · R′_b / max_b R′_b, with ĉ the normalised cue.
//! A folded region's other memories occupy other bands, so the score does not shrink with depth.

pub fn containment(cue: &[f64], region: &[f64]) -> f64 {
    let ct: f64 = cue.iter().sum();
    if ct <= 0.0 || region.is_empty() {
        return 0.0;
    }
    let floor = region.iter().cloned().fold(f64::MAX, f64::min);
    let lifted: Vec<f64> = region.iter().map(|x| (x - floor).max(0.0)).collect();
    let top = lifted.iter().cloned().fold(0.0, f64::max);
    if top <= 0.0 {
        return 0.0;
    }
    cue.iter().zip(&lifted).map(|(c, r)| c / ct * r / top).sum()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn present_absent_and_folded() {
        let floor = vec![0.1; 8];
        let mut region = floor.clone();
        region[1] += 1.0;
        region[2] += 1.0; // memory A = bands 1, 2
        region[5] += 1.0;
        region[6] += 1.0; // memory B = bands 5, 6 (folded in)
        let a = [0.0, 1.0, 1.0, 0.0, 0.0, 0.0, 0.0, 0.0];
        let absent = [1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0];
        assert!((containment(&a, &region) - 1.0).abs() < 1e-12);
        assert!(containment(&absent, &region) < 1e-12);
    }
}
