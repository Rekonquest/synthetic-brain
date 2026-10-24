//! GEO-1 (docs/GEO-1-PRECOMMIT.md): mirror (x → −x, if asked) then rotate by θ about the origin.
//! Mechanism only.

pub fn rotate(p: &[(f64, f64)], theta: f64, mirror: bool) -> Vec<(f64, f64)> {
    let (c, s) = (theta.cos(), theta.sin());
    p.iter().map(|&(x, y)| {
        let x = if mirror { -x } else { x };
        (c * x - s * y, s * x + c * y)
    }).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quarter_turn_and_mirror() {
        let r = rotate(&[(1.0, 0.0)], std::f64::consts::FRAC_PI_2, false);
        assert!(r[0].0.abs() < 1e-12 && (r[0].1 - 1.0).abs() < 1e-12);
        let m = rotate(&[(1.0, 2.0)], 0.0, true);
        assert_eq!(m[0], (-1.0, 2.0));
    }
}
