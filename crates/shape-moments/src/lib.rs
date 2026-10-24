//! GEO-1 (docs/GEO-1-PRECOMMIT.md): the complex moments of a 2D point set about its centroid,
//! m_k = Σ z_i^k with z = (x − x̄) + i(y − ȳ), for k = 0..=kmax. A rotation by θ multiplies m_k by
//! e^{ikθ} (rotation charge k); a reflection conjugates it. Returned as (re, im). Mechanism only.

pub fn moments(p: &[(f64, f64)], kmax: usize) -> Vec<(f64, f64)> {
    let n = p.len() as f64;
    let (cx, cy) = (p.iter().map(|q| q.0).sum::<f64>() / n, p.iter().map(|q| q.1).sum::<f64>() / n);
    let mut m = vec![(0.0, 0.0); kmax + 1];
    for &(x, y) in p {
        let (zr, zi) = (x - cx, y - cy);
        let (mut pr, mut pi) = (1.0, 0.0);
        for mk in m.iter_mut() {
            mk.0 += pr;
            mk.1 += pi;
            let t = pr * zr - pi * zi;
            pi = pr * zi + pi * zr;
            pr = t;
        }
    }
    m
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn charge_k_under_rotation() {
        let p = [(1.0, 0.0), (0.0, 2.0), (-1.5, -0.5), (0.3, -1.0)];
        let th: f64 = 0.7;
        let q: Vec<(f64, f64)> = p.iter().map(|&(x, y)| (th.cos() * x - th.sin() * y, th.sin() * x + th.cos() * y)).collect();
        let (a, b) = (moments(&p, 4), moments(&q, 4));
        for k in 0..=4 {
            let (c, s) = ((k as f64 * th).cos(), (k as f64 * th).sin());
            let rot = (c * a[k].0 - s * a[k].1, s * a[k].0 + c * a[k].1);
            assert!((rot.0 - b[k].0).abs() < 1e-9 && (rot.1 - b[k].1).abs() < 1e-9);
        }
    }
}
