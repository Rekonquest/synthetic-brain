//! SCAR-1 (docs/SCAR-1-PRECOMMIT.md): recall by tap. The scars (conductances) are frozen; the field
//! starts at 0; the cue cells are held at +1 through damping while every other site leaks to 0; the
//! medium conducts along its scars for `steps`, at T 0. The cells' fields are read and ranked. The
//! tap adds nothing to the medium; it only drives and reads it.

use thermo_bath::{damp, exchange_edges};
use thermo_carve::Carving;
use thermo_lattice::Lattice;

/// (the non-cue cells ranked by field, highest first; the last step at which the top `k` changed).
pub fn recall(dims: &[usize], scars: &Carving, kappa: f64, leak: f64, cells: &[usize], cue: &[usize], k: usize, steps: usize) -> (Vec<usize>, usize) {
    let mut l = Lattice::new(dims);
    let d = dims.len();
    let is_cue = |s: usize| cue.contains(&s);
    let rank = |l: &Lattice| {
        let mut r: Vec<usize> = cells.iter().copied().filter(|&s| !is_cue(s)).collect();
        r.sort_by(|a, b| l.e[*b].partial_cmp(&l.e[*a]).unwrap().then(a.cmp(b)));
        r
    };
    let (mut top, mut settled) = (Vec::new(), 0);
    for t in 0..steps {
        damp(&mut l, 0.0, &|s| if is_cue(s) { 1.0 } else { leak }, &|s| if is_cue(s) { 1.0 } else { 0.0 }, &|_| 0.0);
        exchange_edges(&mut l, 0.0, &|s, a| kappa * scars.c[s * d + a], &|_| 0.0, &|_, _| 0.0);
        let mut now: Vec<usize> = rank(&l).into_iter().take(k).collect();
        now.sort();
        if now != top {
            top = now;
            settled = t;
        }
    }
    (rank(&l), settled)
}
