//! SCAR-1 run 2 (docs/SCAR-1-PRECOMMIT.md): competition for the carved medium, borrowed from
//! neuroscience's *synaptic scaling* (homeostatic plasticity: a neuron keeps its total input
//! strength near a set point). Each site's total incident conductance is capped at a budget B: a
//! site over budget gets the factor B / (its total); each edge is scaled by the smaller factor of its
//! two ends and clamped at c_min. Routes that are used keep growing (carving) while the cap takes the
//! share from the unused ones. Mechanism only: B and c_min are the caller's.

use thermo_lattice::Lattice;

/// The edges touching each site, built once: (edge index s·d + axis) for the site's forward and
/// backward edges on every axis.
pub struct Incidence {
    pub edges: Vec<Vec<u32>>,
    pub ends: Vec<(u32, u32)>,
}

impl Incidence {
    pub fn new(l: &Lattice) -> Incidence {
        let d = l.dims.len();
        let mut edges = vec![Vec::new(); l.e.len()];
        let mut ends = Vec::with_capacity(l.e.len() * d);
        for s in 0..l.e.len() {
            for a in 0..d {
                let j = l.forward(s, a);
                ends.push((s as u32, j as u32));
                edges[s].push((s * d + a) as u32);
                edges[j].push((s * d + a) as u32);
            }
        }
        Incidence { edges, ends }
    }
}

/// Cap every site's incident conductance at `budget`.
pub fn cap(c: &mut [f64], inc: &Incidence, budget: f64, c_min: f64) {
    let factor: Vec<f64> = inc.edges.iter().map(|es| { let tot: f64 = es.iter().map(|&e| c[e as usize]).sum(); if tot > budget { budget / tot } else { 1.0 } }).collect();
    for (e, (i, j)) in inc.ends.iter().enumerate() {
        let f = factor[*i as usize].min(factor[*j as usize]);
        if f < 1.0 {
            c[e] = (c[e] * f).max(c_min);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn caps_only_sites_over_budget() {
        let l = Lattice::new(&[4, 4]);
        let inc = Incidence::new(&l);
        let mut c = vec![0.2; l.e.len() * 2];
        cap(&mut c, &inc, 1.0, 0.05);
        assert!(c.iter().all(|&x| x == 0.2)); // 4 edges × 0.2 = 0.8 ≤ 1
        c[0] = 1.0; // site 0's forward-x edge: site 0 and site 1 now total 1.6
        cap(&mut c, &inc, 1.0, 0.05);
        assert!((c[0] - 1.0 / 1.6).abs() < 1e-12);
        let site0: f64 = inc.edges[0].iter().map(|&e| c[e as usize]).sum();
        assert!(site0 <= 1.0 + 1e-12);
    }
}
