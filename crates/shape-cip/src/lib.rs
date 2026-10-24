//! GEO-2 (docs/GEO-2-PRECOMMIT.md): chemistry's Cahn–Ingold–Prelog rule for a 2D point set. Each
//! point's priority is its distance from the centroid, ties broken by its sorted distances to the
//! other points (the next sphere out). The handedness is the turn of the three highest-priority
//! points in priority order, (p2 − p1) × (p3 − p1): positive is one hand, negative the other.
//! Rotation keeps it; reflection flips it. Mechanism only.

pub fn chirality(p: &[(f64, f64)]) -> f64 {
    let n = p.len() as f64;
    let (cx, cy) = (p.iter().map(|q| q.0).sum::<f64>() / n, p.iter().map(|q| q.1).sum::<f64>() / n);
    let key = |i: usize| -> (f64, Vec<f64>) {
        let r = ((p[i].0 - cx).powi(2) + (p[i].1 - cy).powi(2)).sqrt();
        let mut d: Vec<f64> = (0..p.len()).filter(|&j| j != i).map(|j| ((p[i].0 - p[j].0).powi(2) + (p[i].1 - p[j].1).powi(2)).sqrt()).collect();
        d.sort_by(|a, b| b.partial_cmp(a).unwrap());
        (r, d)
    };
    let mut idx: Vec<usize> = (0..p.len()).collect();
    idx.sort_by(|&a, &b| {
        let (ka, kb) = (key(a), key(b));
        kb.0.partial_cmp(&ka.0).unwrap().then_with(|| kb.1.partial_cmp(&ka.1).unwrap())
    });
    let (a, b, c) = (p[idx[0]], p[idx[1]], p[idx[2]]);
    (b.0 - a.0) * (c.1 - a.1) - (b.1 - a.1) * (c.0 - a.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rotation_keeps_and_mirror_flips() {
        let p = [(3.0, 0.0), (0.0, 2.0), (-1.0, -1.0), (0.2, 0.1)];
        let th: f64 = 2.3;
        let r: Vec<(f64, f64)> = p.iter().map(|&(x, y)| (th.cos() * x - th.sin() * y, th.sin() * x + th.cos() * y)).collect();
        let m: Vec<(f64, f64)> = p.iter().map(|&(x, y)| (-x, y)).collect();
        assert!(chirality(&p) * chirality(&r) > 0.0);
        assert!(chirality(&p) * chirality(&m) < 0.0);
    }
}
