//! GEO-1 (docs/GEO-1-PRECOMMIT.md): the sorted multiset of all pairwise distances of a point set,
//! conserved by every rotation, translation and reflection. Mechanism only.

pub fn sorted_distances(p: &[(f64, f64)]) -> Vec<f64> {
    let mut d = Vec::new();
    for i in 0..p.len() {
        for j in i + 1..p.len() {
            d.push(((p[i].0 - p[j].0).powi(2) + (p[i].1 - p[j].1).powi(2)).sqrt());
        }
    }
    d.sort_by(|a, b| a.partial_cmp(b).unwrap());
    d
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_right_triangle() {
        assert_eq!(sorted_distances(&[(0.0, 0.0), (3.0, 0.0), (0.0, 4.0)]), vec![3.0, 4.0, 5.0]);
    }
}
