//! GEO-1 (docs/GEO-1-PRECOMMIT.md): spontaneous symmetry breaking for a 2D point set. Centre it,
//! rotate its principal axis (largest second moment) onto x, choose that axis's sign so the third
//! moment along x (Σ x³) is positive, and reflect y if the third moment along y (Σ y³) is negative.
//! Returns (the canonical points, whether a reflection was applied, the eigenvalue ratio
//! λ_min/λ_max: near 1 means the symmetry is barely broken). Mechanism only.

pub fn canon(p: &[(f64, f64)]) -> (Vec<(f64, f64)>, bool, f64) {
    let n = p.len() as f64;
    let (cx, cy) = (p.iter().map(|q| q.0).sum::<f64>() / n, p.iter().map(|q| q.1).sum::<f64>() / n);
    let c: Vec<(f64, f64)> = p.iter().map(|q| (q.0 - cx, q.1 - cy)).collect();
    let (mut sxx, mut syy, mut sxy) = (0.0, 0.0, 0.0);
    for &(x, y) in &c {
        sxx += x * x;
        syy += y * y;
        sxy += x * y;
    }
    let theta = 0.5 * (2.0 * sxy).atan2(sxx - syy);
    let tr = sxx + syy;
    let disc = ((sxx - syy).powi(2) + 4.0 * sxy * sxy).sqrt();
    let ratio = if tr + disc > 0.0 { (tr - disc) / (tr + disc) } else { 1.0 };
    let (co, si) = (theta.cos(), theta.sin());
    let mut q: Vec<(f64, f64)> = c.iter().map(|&(x, y)| (co * x + si * y, -si * x + co * y)).collect();
    if q.iter().map(|v| v.0.powi(3)).sum::<f64>() < 0.0 {
        q.iter_mut().for_each(|v| *v = (-v.0, -v.1));
    }
    let mirrored = q.iter().map(|v| v.1.powi(3)).sum::<f64>() < 0.0;
    if mirrored {
        q.iter_mut().for_each(|v| v.1 = -v.1);
    }
    (q, mirrored, ratio)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_invariant_and_mirror_flagged() {
        let p = [(2.0, 0.1), (-1.0, 0.3), (0.5, -0.9), (-1.5, 0.5), (0.0, 1.2)];
        let (a, ma, _) = canon(&p);
        let th: f64 = 1.1;
        let r: Vec<(f64, f64)> = p.iter().map(|&(x, y)| (th.cos() * x - th.sin() * y + 3.0, th.sin() * x + th.cos() * y - 1.0)).collect();
        let (b, mb, _) = canon(&r);
        assert_eq!(ma, mb);
        for (u, v) in a.iter().zip(&b) {
            assert!((u.0 - v.0).abs() < 1e-9 && (u.1 - v.1).abs() < 1e-9);
        }
        let m: Vec<(f64, f64)> = p.iter().map(|&(x, y)| (-x, y)).collect();
        assert_ne!(canon(&m).1, ma);
    }
}
