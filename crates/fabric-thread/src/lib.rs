//! Spiderweb fabric — the strongest thread (FABRIC-1, reporting): the path from any source to a sink
//! with the largest product of conductances, over a small directed graph. Mechanism only.

/// `edges[(from, to)] = conductance`. Returns the strongest path ending at `sink`, and its strength.
pub fn strongest(edges: &[(usize, usize, f64)], nodes: usize, sink: usize) -> (Vec<usize>, f64) {
    // Bellman-Ford style over a DAG-sized graph: best[v] = best product into v.
    let mut best: Vec<(f64, Vec<usize>)> = (0..nodes).map(|v| (1.0, vec![v])).collect();
    for _ in 0..nodes {
        for &(a, b, g) in edges {
            let cand = best[a].0 * g;
            if cand > best[b].0 * if best[b].1.len() == 1 { 0.0 } else { 1.0 } && !best[a].1.contains(&b) {
                let mut p = best[a].1.clone();
                p.push(b);
                best[b] = (cand, p);
            }
        }
    }
    (best[sink].1.clone(), if best[sink].1.len() > 1 { best[sink].0 } else { 0.0 })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn finds_the_strongest_path() {
        // 0 -> 2 -> 3 strong; 1 -> 3 weak.
        let edges = [(0, 2, 0.9), (2, 3, 0.8), (1, 3, 0.2), (0, 3, 0.1)];
        let (p, s) = strongest(&edges, 4, 3);
        assert_eq!(p, vec![0, 2, 3]);
        assert!((s - 0.72).abs() < 1e-12);
    }
}
